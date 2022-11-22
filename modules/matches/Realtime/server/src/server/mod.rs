use fnv::FnvHashSet;
use std::cell::RefCell;
use std::net::UdpSocket;
use std::ops::Add;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{io, thread};

use cheetah_matches_realtime_common::protocol::others::member_id::MemberAndRoomId;
use cheetah_matches_realtime_common::room::{RoomId, RoomMemberId};

use crate::room::command::ServerCommandError;
use crate::room::template::config::MemberTemplate;
use crate::server::manager::{ChannelTask, ManagementTask, ManagementTaskResult, TaskExecutionError};
use crate::server::measurers::Measurers;
use crate::server::network::NetworkLayer;
use crate::server::rooms::{RoomNotFoundError, Rooms};

pub mod manager;
pub mod measurers;
pub mod network;
pub mod rooms;

///
/// Собственно сетевой сервер, запускается в отдельном потоке, обрабатывает сетевые команды,
/// поддерживает одновременно несколько комнат
///
pub struct RoomsServer {
	network_layer: NetworkLayer,
	rooms: Rooms,
	receiver: Receiver<ChannelTask>,
	halt_signal: Arc<AtomicBool>,
	time_offset: Option<Duration>,
	measurers: Rc<RefCell<Measurers>>,
	plugin_names: FnvHashSet<String>,
}

impl RoomsServer {
	pub(crate) fn new(
		socket: UdpSocket,
		receiver: Receiver<ChannelTask>,
		halt_signal: Arc<AtomicBool>,
		plugin_names: FnvHashSet<String>,
	) -> Result<Self, io::Error> {
		let measures = Rc::new(RefCell::new(Measurers::new(prometheus::default_registry())));
		Ok(Self {
			network_layer: NetworkLayer::new(socket, measures.clone())?,
			rooms: Rooms::new(measures.clone(), plugin_names.clone()),
			receiver,
			halt_signal,
			time_offset: None,
			measurers: measures,
			plugin_names,
		})
	}

	pub fn run(mut self) {
		while !self.halt_signal.load(Ordering::Relaxed) {
			let mut now = Instant::now();
			if let Some(time_offset) = self.time_offset {
				now = now.add(time_offset);
			}
			self.network_layer.cycle(&mut self.rooms, now);
			self.execute_management_tasks(now);
			self.measurers.borrow_mut().on_server_cycle(now.elapsed());
			thread::sleep(Duration::from_millis(1));
		}
	}

	fn execute_management_tasks(&mut self, now: Instant) {
		while let Ok(ChannelTask { task, sender }) = self.receiver.try_recv() {
			let res = self.execute_task(task, now);
			if let Err(e) = sender.send(res) {
				tracing::error!("error send response {:?}", e);
			}
		}
	}

	fn execute_task(&mut self, task: ManagementTask, now: Instant) -> Result<ManagementTaskResult, TaskExecutionError> {
		let res = match task {
			ManagementTask::CreateRoom(template) => ManagementTaskResult::CreateRoom(self.rooms.create_room(template)),
			ManagementTask::DeleteRoom(room_id) => self
				.delete_room(room_id)
				.map(|_| ManagementTaskResult::DeleteRoom)
				.map_err(TaskExecutionError::RoomNotFound)?,
			ManagementTask::CreateMember(room_id, user_template) => self
				.register_user(room_id, user_template, now)
				.map(ManagementTaskResult::CreateMember)
				.map_err(TaskExecutionError::RoomNotFound)?,
			ManagementTask::DeleteMember(id) => self
				.delete_member(id)
				.map(|_| ManagementTaskResult::DeleteMember)
				.map_err(TaskExecutionError::ServerCommandError)?,
			ManagementTask::Dump(room_id) => self
				.rooms
				.room_by_id
				.get(&room_id)
				.map(|room| ManagementTaskResult::Dump(room.into()))
				.ok_or(TaskExecutionError::RoomNotFound(RoomNotFoundError(room_id)))?,
			ManagementTask::GetRooms => ManagementTaskResult::GetRooms(self.rooms.room_by_id.keys().copied().collect()),
			ManagementTask::CommandTracerSessionTask(room_id, task) => self
				.rooms
				.room_by_id
				.get_mut(&room_id)
				.map(|room| {
					room.command_trace_session.clone().borrow_mut().execute_task(task);
					ManagementTaskResult::CommandTracerSessionTask
				})
				.ok_or(TaskExecutionError::RoomNotFound(RoomNotFoundError(room_id)))?,
			ManagementTask::PutForwardedCommandConfig(room_id, config) => self
				.rooms
				.room_by_id
				.get_mut(&room_id)
				.map(|room| {
					room.put_forwarded_command_config(config);
					ManagementTaskResult::PutForwardedCommandConfig
				})
				.ok_or(TaskExecutionError::RoomNotFound(RoomNotFoundError(room_id)))?,
			ManagementTask::MarkRoomAsReady(room_id, plugin_name) => self.mark_room_as_ready(room_id, plugin_name)?,
			ManagementTask::GetRoomInfo(room_id) => self
				.rooms
				.room_by_id
				.get(&room_id)
				.map(|room| ManagementTaskResult::GetRoomInfo(room.get_info()))
				.ok_or(TaskExecutionError::RoomNotFound(RoomNotFoundError(room_id)))?,
		};
		Ok(res)
	}

	fn mark_room_as_ready(&mut self, room_id: RoomId, plugin_name: String) -> Result<ManagementTaskResult, TaskExecutionError> {
		if let Some(room) = self.rooms.room_by_id.get_mut(&room_id) {
			if self.plugin_names.contains(&plugin_name) {
				room.mark_room_as_ready(&plugin_name);
				Ok(ManagementTaskResult::MarkRoomAsReady)
			} else {
				Err(TaskExecutionError::UnknownPluginName(plugin_name))
			}
		} else {
			Err(TaskExecutionError::RoomNotFound(RoomNotFoundError(room_id)))
		}
	}

	fn register_user(&mut self, room_id: RoomId, user_template: MemberTemplate, now: Instant) -> Result<RoomMemberId, RoomNotFoundError> {
		let room_member_id = self.rooms.register_user(room_id, user_template.clone())?;
		self.network_layer.register_user(now, room_id, room_member_id, user_template);
		Ok(room_member_id)
	}

	/// удалить комнату с севрера и закрыть соединение со всеми пользователями
	fn delete_room(&mut self, room_id: RoomId) -> Result<(), RoomNotFoundError> {
		let room = self.rooms.take_room(&room_id)?;
		let ids = room.members.into_keys().map(|member_id| MemberAndRoomId { member_id, room_id });
		self.network_layer.disconnect_users(ids);
		Ok(())
	}

	/// закрыть соединение с пользователем и удалить его из комнаты
	fn delete_member(&mut self, id: MemberAndRoomId) -> Result<(), ServerCommandError> {
		self.network_layer.disconnect_users([id].into_iter());
		self.rooms.user_disconnected(&id)
	}
}
