use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use thiserror::Error;

use cheetah_matches_realtime_common::room::{RoomId, RoomMemberId};

use crate::debug::proto::admin;
use crate::debug::tracer::TracerSessionCommand;
use crate::room::template::config::{MemberTemplate, RoomTemplate};
use crate::server::manager::ManagementTask::TimeOffset;
use crate::server::rooms::RegisterUserError;
use crate::server::RoomsServer;

///
/// Управление сервером
/// - запуск сервера в отдельном потоке
/// - связь с сервером через Sender
///
pub struct RoomsServerManager {
	handler: Option<JoinHandle<()>>,
	sender: Sender<ManagementTask>,
	halt_signal: Arc<AtomicBool>,
	pub created_room_counter: usize,
}

pub enum ManagementTask {
	CreateRoom(RoomTemplate, Sender<RoomId>),
	CreateMember(RoomId, MemberTemplate, Sender<Result<RoomMemberId, RegisterUserError>>),
	///
	/// Смещение текущего времени для тестирования
	///
	TimeOffset(Duration),
	Dump(RoomId, Sender<Result<admin::DumpResponse, String>>),
	GetRooms(Sender<Vec<RoomId>>),
	CommandTracerSessionTask(RoomId, TracerSessionCommand, Sender<Result<(), CommandTracerSessionTaskError>>),
}

#[derive(Debug)]
pub enum CommandTracerSessionTaskError {
	RoomNotFound(RoomId),
	RecvTimeoutError,
}

#[derive(Debug, Error)]
pub enum RegisterRoomRequestError {
	#[error("ChannelError {0}")]
	ChannelError(RecvTimeoutError),
}

#[derive(Debug, Error)]
pub enum CreateMemberRequestError {
	#[error("ChannelError {0}")]
	ChannelError(RecvTimeoutError),
	#[error("Error {0}")]
	Error(RegisterUserError),
}

impl Drop for RoomsServerManager {
	fn drop(&mut self) {
		self.halt_signal.store(true, Ordering::Relaxed);
		println!("Drop RoomsServerManager");
	}
}

impl RoomsServerManager {
	pub fn new(socket: UdpSocket) -> Self {
		let (sender, receiver) = std::sync::mpsc::channel();
		let halt_signal = Arc::new(AtomicBool::new(false));
		let cloned_halt_signal = halt_signal.clone();
		let handler = thread::Builder::new()
			.name(format!("server({:?})", socket.local_addr().unwrap()))
			.spawn(move || {
				println!("RUN SERVER {:?}", socket);
				RoomsServer::new(socket, receiver, halt_signal).run();
			})
			.unwrap();
		Self {
			handler: Some(handler),
			sender,
			halt_signal: cloned_halt_signal,
			created_room_counter: 0,
		}
	}

	pub fn get_rooms(&self) -> Result<Vec<RoomId>, String> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(ManagementTask::GetRooms(sender)).unwrap();
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(rooms) => Ok(rooms),
			Err(e) => Err(format!("{:?}", e)),
		}
	}

	///
	/// Выполнить задачу в CommandTracerSessions конкретной комнаты
	/// Подход с вложенным enum для отдельного класса задач применяется для изолирования функционала
	///
	pub fn execute_command_trace_sessions_task(&self, room_id: RoomId, task: TracerSessionCommand) -> Result<(), CommandTracerSessionTaskError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender
			.send(ManagementTask::CommandTracerSessionTask(room_id, task, sender))
			.unwrap_or_else(|_| panic!("{}", expect_send_msg("CommandTracerSessionTask")));
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(r) => match r {
				Ok(_) => Ok(()),
				Err(e) => Err(e),
			},
			Err(_e) => Err(CommandTracerSessionTaskError::RecvTimeoutError),
		}
	}

	pub fn get_halt_signal(&self) -> Arc<AtomicBool> {
		self.halt_signal.clone()
	}

	pub fn create_room(&mut self, template: RoomTemplate) -> Result<RoomId, RegisterRoomRequestError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender
			.send(ManagementTask::CreateRoom(template, sender))
			.unwrap_or_else(|_| panic!("{}", expect_send_msg("RegisterRoom")));
		self.created_room_counter += 1;
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(room_id) => {
				tracing::info!("[server] create room({:?})", room_id);
				Ok(room_id)
			}
			Err(e) => {
				tracing::error!("[server] fail create room");
				Err(RegisterRoomRequestError::ChannelError(e))
			}
		}
	}

	pub fn create_member(&mut self, room_id: RoomId, template: MemberTemplate) -> Result<RoomMemberId, CreateMemberRequestError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender
			.send(ManagementTask::CreateMember(room_id, template.clone(), sender))
			.unwrap_or_else(|_| panic!("{}", expect_send_msg("RegisterUser")));
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(r) => match r {
				Ok(user_id) => {
					tracing::info!("[server] create member({:?}) in room ({:?})", user_id, room_id);
					Ok(user_id)
				}
				Err(e) => {
					tracing::error!(
						"[server] fail create member ({:?}) in room ({:?}) with error {:?}",
						template.private_key,
						room_id,
						e
					);
					Err(CreateMemberRequestError::Error(e))
				}
			},
			Err(e) => {
				tracing::error!(
					"[server] fail create user ({:?}) in room ({:?}) with error {:?}",
					template.private_key,
					room_id,
					e
				);
				Err(CreateMemberRequestError::ChannelError(e))
			}
		}
	}

	pub fn set_time_offset(&self, duration: Duration) {
		self.sender
			.send(TimeOffset(duration))
			.unwrap_or_else(|_| panic!("{}", expect_send_msg("TimeOffset")));
	}

	pub fn join(&mut self) {
		self.handler.take().unwrap().join().unwrap();
	}

	pub fn dump(&self, room_id: u64) -> Result<admin::DumpResponse, String> {
		let (sender, receiver) = std::sync::mpsc::channel();
		match self.sender.send(ManagementTask::Dump(room_id, sender)) {
			Ok(_) => match receiver.recv() {
				Ok(result) => result,
				Err(e) => Err(format!("{:?}", e)),
			},
			Err(e) => Err(format!("{:?}", e)),
		}
	}

	pub fn shutdown(&mut self) {
		self.halt_signal.store(true, Ordering::Relaxed);
	}
}

fn expect_send_msg(task: &str) -> String {
	format!("Can not send {} to relay thread, possible relay thread is dead", task)
}

#[cfg(test)]
mod test {
	use cheetah_matches_realtime_common::network::bind_to_free_socket;

	use crate::room::template::config::RoomTemplate;
	use crate::server::manager::RoomsServerManager;

	#[test]
	fn should_increment_created_room_count() {
		let mut server = RoomsServerManager::new(bind_to_free_socket().unwrap().0);
		server.create_room(RoomTemplate::default()).unwrap();
		assert_eq!(server.created_room_counter, 1);
	}

	#[test]
	fn should_get_rooms() {
		let mut server = RoomsServerManager::new(bind_to_free_socket().unwrap().0);
		let room_id = server.create_room(RoomTemplate::default()).unwrap();
		let rooms = server.get_rooms().unwrap();
		assert_eq!(rooms, vec![room_id]);
	}
}
