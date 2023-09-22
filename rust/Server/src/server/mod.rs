use crate::room::command::ServerCommandError;
use crate::room::template::config::MemberTemplate;
use crate::server::manager::{ManagementTask, ManagementTaskChannel, ManagementTaskExecutionError, ManagementTaskResult, RoomMembersCount};
use crate::server::measurer::Measurer;
use crate::server::network::Network;
use crate::server::room_registry::{RoomNotFoundError, Rooms};
use cheetah_game_realtime_protocol::coniguration::ProtocolConfiguration;
use cheetah_game_realtime_protocol::disconnect::command::DisconnectByCommandReason;
use cheetah_game_realtime_protocol::others::member_id::MemberAndRoomId;
use cheetah_game_realtime_protocol::{RoomId, RoomMemberId};
use std::cell::RefCell;
use std::net::UdpSocket;
use std::ops::Add;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{io, iter, thread};

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
	rooms: Rooms,
	management_task_receiver: Receiver<ManagementTaskChannel>,
	halt_signal: Arc<AtomicBool>,
	time_offset: Option<Duration>,
	measurer: RefCell<Measurer>,
}

impl Server {
	pub(crate) fn new(
		socket: UdpSocket,
		management_task_receiver: Receiver<ManagementTaskChannel>,
		halt_signal: Arc<AtomicBool>,
		protocol_configuration: ProtocolConfiguration,
	) -> Result<Self, io::Error> {
		let measurer = Measurer::new(prometheus::default_registry()).into();
		Ok(Self {
			network: Network::new(socket, protocol_configuration)?,
			rooms: Rooms::new(),
			management_task_receiver,
			halt_signal,
			time_offset: None,
			measurer,
		})
	}

	pub fn run(mut self) {
		while !self.halt_signal.load(Ordering::Relaxed) {
			let now = self.get_start_cycle_time();
			self.network.cycle(&mut self.rooms, now);
			self.execute_management_tasks(now);
			self.measurer.borrow_mut().measure_cycle(&self.network, &self.rooms, &now);
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
			ManagementTask::CreateRoom(template) => ManagementTaskResult::CreateRoom(self.rooms.create_room(template)),
			ManagementTask::DeleteRoom(room_id) => self.delete_room(room_id).map(|_| ManagementTaskResult::DeleteRoom)?,
			ManagementTask::CreateMember(room_id, member_template) => self.register_member(room_id, member_template, now).map(ManagementTaskResult::CreateMember)?,
			ManagementTask::DeleteMember(id) => self.delete_member(id).map(|_| ManagementTaskResult::DeleteMember)?,
			ManagementTask::Dump(room_id) => ManagementTaskResult::Dump(self.rooms.get(&room_id).cloned()),
			ManagementTask::GetRooms => ManagementTaskResult::GetRooms(self.rooms.rooms().map(|r| r.0).copied().collect()),
			ManagementTask::GetRoomsMemberCount => ManagementTaskResult::GetRoomsMemberCount(
				self.rooms
					.rooms()
					.map(|(room_id, room)| RoomMembersCount {
						room_id: *room_id,
						members: room.members.len(),
						connected_members: room.members.iter().filter(|p| p.1.connected).count(),
					})
					.collect(),
			),
			ManagementTask::GetCreatedRoomsCount => ManagementTaskResult::GetCreatedRoomsCount(self.rooms.created_rooms_count),
		};
		Ok(res)
	}

	fn register_member(&mut self, room_id: RoomId, member_template: MemberTemplate, now: Instant) -> Result<RoomMemberId, RoomNotFoundError> {
		let room_member_id = self.rooms.register_member(room_id, member_template.clone())?;
		self.network.register_member(now, room_id, room_member_id, member_template);
		Ok(room_member_id)
	}

	/// удалить комнату с сервера и закрыть соединение со всеми пользователями
	fn delete_room(&mut self, room_id: RoomId) -> Result<(), RoomNotFoundError> {
		let room = self.rooms.force_remove_room(&room_id)?;
		tracing::info!("Delete room {:?}, counts rooms after {:?}", room_id, self.rooms.rooms().len());
		let ids = room.members.into_keys().map(|member_id| MemberAndRoomId { member_id, room_id });
		self.network.disconnect_members(ids, DisconnectByCommandReason::RoomDeleted);
		Ok(())
	}

	/// закрыть соединение с пользователем и удалить его из комнаты
	fn delete_member(&mut self, id: MemberAndRoomId) -> Result<(), ServerCommandError> {
		self.network.disconnect_members(iter::once(id), DisconnectByCommandReason::MemberDeleted);
		self.rooms.member_disconnected(&id)
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
