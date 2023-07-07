use std::cell::RefCell;
use std::net::UdpSocket;
use std::ops::Add;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{io, iter, thread};

use fnv::FnvHashSet;

use cheetah_protocol::coniguration::ProtocolConfiguration;
use cheetah_protocol::disconnect::command::DisconnectByCommandReason;
use cheetah_protocol::others::member_id::MemberAndRoomId;
use cheetah_protocol::{RoomId, RoomMemberId};

use crate::room::command::ServerCommandError;
use crate::room::template::config::{MemberTemplate, Permissions};
use crate::server::manager::{ManagementTask, ManagementTaskChannel, ManagementTaskExecutionError, ManagementTaskResult, RoomMembersCount};
use crate::server::measurer::Measurer;
use crate::server::network::Network;
use crate::server::room_registry::{RoomNotFoundError, RoomRegistry};

pub mod manager;
pub mod measurer;
pub mod network;
pub mod room_registry;

///
/// Собственно сетевой сервер, запускается в отдельном потоке, обрабатывает сетевые команды,
/// поддерживает одновременно несколько комнат
///
pub struct Server {
	network: Network,
	room_registry: RoomRegistry,
	management_task_receiver: Receiver<ManagementTaskChannel>,
	halt_signal: Arc<AtomicBool>,
	time_offset: Option<Duration>,
	measurer: RefCell<Measurer>,
	plugin_names: FnvHashSet<String>,
}

impl Server {
	pub(crate) fn new(
		socket: UdpSocket,
		management_task_receiver: Receiver<ManagementTaskChannel>,
		halt_signal: Arc<AtomicBool>,
		plugin_names: FnvHashSet<String>,
		protocol_configuration: ProtocolConfiguration,
	) -> Result<Self, io::Error> {
		let measurer = Measurer::new(prometheus::default_registry()).into();
		Ok(Self {
			network: Network::new(socket, protocol_configuration)?,
			room_registry: RoomRegistry::new(plugin_names.clone()),
			management_task_receiver,
			halt_signal,
			time_offset: None,
			measurer,
			plugin_names,
		})
	}

	pub fn run(mut self) {
		while !self.halt_signal.load(Ordering::Relaxed) {
			let now = self.get_start_cycle_time();
			self.network.cycle(&mut self.room_registry, now);
			self.execute_management_tasks(now);
			self.measurer.borrow_mut().measure_cycle(&self.network, &self.room_registry, &now);
			Self::assert_execution_time(now);
			Self::sleep();
		}
	}

	fn get_start_cycle_time(&self) -> Instant {
		let mut now = Instant::now();
		if let Some(time_offset) = self.time_offset {
			now = now.add(time_offset);
		}
		now
	}

	fn execute_management_tasks(&mut self, now: Instant) {
		while let Ok(ManagementTaskChannel { task, sender }) = self.management_task_receiver.try_recv() {
			let res = self.execute_task(task, now);
			if let Err(e) = sender.send(res) {
				tracing::error!("error send response {:?} with task {:?}", e, e.0);
			}
		}
	}

	fn execute_task(&mut self, task: ManagementTask, now: Instant) -> Result<ManagementTaskResult, ManagementTaskExecutionError> {
		let res = match task {
			ManagementTask::CreateRoom(template) => ManagementTaskResult::CreateRoom(self.room_registry.create_room(template)),
			ManagementTask::DeleteRoom(room_id) => self.delete_room(room_id).map(|_| ManagementTaskResult::DeleteRoom)?,
			ManagementTask::CreateMember(room_id, member_template) => self.register_member(room_id, member_template, now).map(ManagementTaskResult::CreateMember)?,
			ManagementTask::DeleteMember(id) => self.delete_member(id).map(|_| ManagementTaskResult::DeleteMember)?,
			ManagementTask::Dump(room_id) => self
				.room_registry
				.get(&room_id)
				.map(|room| ManagementTaskResult::Dump(room.into()))
				.ok_or(ManagementTaskExecutionError::RoomNotFound(RoomNotFoundError(room_id)))?,
			ManagementTask::GetRooms => ManagementTaskResult::GetRooms(self.room_registry.rooms().map(|r| r.0).copied().collect()),
			ManagementTask::PutForwardedCommandConfig(room_id, config) => self
				.room_registry
				.get_mut(&room_id)
				.map(|room| {
					room.put_forwarded_command_config(config);
					ManagementTaskResult::PutForwardedCommandConfig
				})
				.ok_or(ManagementTaskExecutionError::RoomNotFound(RoomNotFoundError(room_id)))?,
			ManagementTask::MarkRoomAsReady(room_id, plugin_name) => self.mark_room_as_ready(room_id, plugin_name)?,
			ManagementTask::GetRoomInfo(room_id) => self
				.room_registry
				.get(&room_id)
				.map(|room| ManagementTaskResult::GetRoomInfo(room.get_info()))
				.ok_or(ManagementTaskExecutionError::RoomNotFound(RoomNotFoundError(room_id)))?,
			ManagementTask::UpdateRoomPermissions(room_id, permissions) => self.update_room_permissions(room_id, &permissions)?,
			ManagementTask::GetRoomsMemberCount => ManagementTaskResult::GetRoomsMemberCount(
				self.room_registry
					.rooms()
					.map(|(room_id, room)| RoomMembersCount {
						room_id: *room_id,
						members: room.members.len(),
						connected_members: room.members.iter().filter(|p| p.1.connected).count(),
					})
					.collect(),
			),
		};
		Ok(res)
	}

	fn mark_room_as_ready(&mut self, room_id: RoomId, plugin_name: String) -> Result<ManagementTaskResult, ManagementTaskExecutionError> {
		if let Some(room) = self.room_registry.get_mut(&room_id) {
			if self.plugin_names.contains(&plugin_name) {
				room.mark_room_as_ready(&plugin_name);
				Ok(ManagementTaskResult::MarkRoomAsReady)
			} else {
				Err(ManagementTaskExecutionError::UnknownPluginName(plugin_name))
			}
		} else {
			Err(ManagementTaskExecutionError::RoomNotFound(RoomNotFoundError(room_id)))
		}
	}

	fn register_member(&mut self, room_id: RoomId, member_template: MemberTemplate, now: Instant) -> Result<RoomMemberId, RoomNotFoundError> {
		let room_member_id = self.room_registry.register_member(room_id, member_template.clone())?;
		self.network.register_member(now, room_id, room_member_id, member_template);
		Ok(room_member_id)
	}

	/// удалить комнату с сервера и закрыть соединение со всеми пользователями
	fn delete_room(&mut self, room_id: RoomId) -> Result<(), RoomNotFoundError> {
		let room = self.room_registry.force_remove_room(&room_id)?;
		let ids = room.members.into_keys().map(|member_id| MemberAndRoomId { member_id, room_id });
		self.network.disconnect_members(ids, DisconnectByCommandReason::RoomDeleted);
		Ok(())
	}

	/// закрыть соединение с пользователем и удалить его из комнаты
	fn delete_member(&mut self, id: MemberAndRoomId) -> Result<(), ServerCommandError> {
		self.network.disconnect_members(iter::once(id), DisconnectByCommandReason::MemberDeleted);
		self.room_registry.member_disconnected(&id)
	}

	fn update_room_permissions(&mut self, room_id: RoomId, permissions: &Permissions) -> Result<ManagementTaskResult, ManagementTaskExecutionError> {
		self.room_registry
			.get_mut(&room_id)
			.map(|room| {
				room.update_permissions(permissions);
				Ok(ManagementTaskResult::UpdateRoomPermissions)
			})
			.ok_or(ManagementTaskExecutionError::RoomNotFound(RoomNotFoundError(room_id)))?
	}
	fn sleep() {
		thread::sleep(Duration::from_millis(1));
	}

	fn assert_execution_time(now: Instant) {
		if now.elapsed() > Duration::from_secs(1) {
			tracing::error!("slow cycle, time ={:?} ", now.elapsed());
		}
	}
}
