use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use cheetah_matches_relay_common::room::{RoomId, UserId};

use crate::debug::tracer::CommandTracerSessionsTask;
use crate::room::template::config::{RoomTemplate, UserTemplate};
use crate::server::dump::ServerDump;
use crate::server::manager::ManagementTask::TimeOffset;
use crate::server::relay::Relay;
use crate::server::rooms::RegisterUserError;

///
/// Управление сервером
/// - запуск сервера в отдельном потоке
/// - связь с сервером через Sender
///
pub struct RelayManager {
	handler: Option<JoinHandle<()>>,
	sender: Sender<ManagementTask>,
	halt_signal: Arc<AtomicBool>,
	pub created_room_counter: usize,
}

pub enum ManagementTask {
	RegisterRoom(RoomTemplate, Sender<RoomId>),
	RegisterUser(RoomId, UserTemplate, Sender<Result<UserId, RegisterUserError>>),
	///
	/// Смещение текущего времени для тестирования
	///
	TimeOffset(Duration),

	///
	/// Скопировать состояние сервера для отладки
	///
	Dump(Sender<ServerDump>),
	///
	/// Запросить список комнат
	///
	GetRooms(Sender<Vec<RoomId>>),
	///
	/// Выполнить задачу для трассировщика команд
	CommandTracerSessionTask(RoomId, CommandTracerSessionsTask, Sender<Result<(), CommandTracerSessionTaskError>>),
}

#[derive(Debug)]
pub enum CommandTracerSessionTaskError {
	RoomNotFound,
	RecvTimeoutError,
}

#[derive(Debug)]
pub enum RegisterRoomRequestError {
	ChannelError(RecvTimeoutError),
}

#[derive(Debug)]
pub enum RegisterUserRequestError {
	ChannelError(RecvTimeoutError),
	Error(RegisterUserError),
}

impl Drop for RelayManager {
	fn drop(&mut self) {
		self.halt_signal.store(true, Ordering::Relaxed);
	}
}

impl RelayManager {
	pub fn new(socket: UdpSocket) -> Self {
		let (sender, receiver) = std::sync::mpsc::channel();
		let halt_signal = Arc::new(AtomicBool::new(false));
		let cloned_halt_signal = halt_signal.clone();
		let handler = thread::Builder::new()
			.name(format!("server({:?})", socket.local_addr().unwrap()))
			.spawn(move || {
				Relay::new(socket, receiver, halt_signal).run();
			})
			.unwrap();
		Self {
			handler: Option::Some(handler),
			sender,
			halt_signal: cloned_halt_signal,
			created_room_counter: 0,
		}
	}

	pub fn get_rooms(&self) -> Result<Vec<RoomId>, String> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(ManagementTask::GetRooms(sender)).unwrap();
		match receiver.recv_timeout(Duration::from_millis(100)) {
			Ok(rooms) => Result::Ok(rooms),
			Err(e) => Result::Err(format!("{:?}", e).to_string()),
		}
	}

	///
	/// Выполнить задачу в CommandTracerSessions конкретной комнаты
	/// Подход с вложенным enum для отдельного класса задач применяется для изолирования функционала
	///
	pub fn execute_command_trace_sessions_task(&self, room_id: RoomId, task: CommandTracerSessionsTask) -> Result<(), CommandTracerSessionTaskError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(ManagementTask::CommandTracerSessionTask(room_id, task, sender)).unwrap();
		match receiver.recv_timeout(Duration::from_millis(100)) {
			Ok(r) => match r {
				Ok(_) => Result::Ok(()),
				Err(e) => Result::Err(e),
			},
			Err(_e) => Result::Err(CommandTracerSessionTaskError::RecvTimeoutError),
		}
	}

	pub fn get_halt_signal(&self) -> Arc<AtomicBool> {
		self.halt_signal.clone()
	}

	pub fn register_room(&mut self, template: RoomTemplate) -> Result<RoomId, RegisterRoomRequestError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(ManagementTask::RegisterRoom(template, sender)).unwrap();
		self.created_room_counter += 1;
		match receiver.recv_timeout(Duration::from_millis(100)) {
			Ok(room_id) => {
				log::info!("[server] create room({:?})", room_id);
				Result::Ok(room_id)
			}
			Err(e) => {
				log::error!("[server] fail create room");
				Result::Err(RegisterRoomRequestError::ChannelError(e))
			}
		}
	}

	pub fn register_user(&mut self, room_id: RoomId, template: UserTemplate) -> Result<UserId, RegisterUserRequestError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(ManagementTask::RegisterUser(room_id, template.clone(), sender)).unwrap();
		match receiver.recv_timeout(Duration::from_millis(100)) {
			Ok(r) => match r {
				Ok(user_id) => {
					log::info!("[server] create user({:?}) in room ({:?})", user_id, room_id);
					Result::Ok(user_id)
				}
				Err(e) => {
					log::error!(
						"[server] fail create user ({:?}) in room ({:?}) with error {:?}",
						template.private_key,
						room_id,
						e
					);
					Result::Err(RegisterUserRequestError::Error(e))
				}
			},
			Err(e) => {
				log::error!(
					"[server] fail create user ({:?}) in room ({:?}) with error {:?}",
					template.private_key,
					room_id,
					e
				);
				Result::Err(RegisterUserRequestError::ChannelError(e))
			}
		}
	}

	pub fn set_time_offset(&self, duration: Duration) {
		self.sender.send(TimeOffset(duration)).unwrap();
	}

	pub fn join(&mut self) {
		self.handler.take().unwrap().join().unwrap();
	}

	pub fn dump(&self) -> Result<ServerDump, ()> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(ManagementTask::Dump(sender)).unwrap();
		receiver.recv().map_err(|_| ())
	}
}

#[cfg(test)]
mod test {
	use cheetah_matches_relay_common::network::bind_to_free_socket;

	use crate::room::template::config::RoomTemplate;
	use crate::server::manager::RelayManager;

	#[test]
	fn should_increment_created_room_count() {
		let mut server = RelayManager::new(bind_to_free_socket().unwrap().0);
		server.register_room(RoomTemplate::default()).unwrap();
		assert_eq!(server.created_room_counter, 1);
	}

	#[test]
	fn should_get_rooms() {
		let mut server = RelayManager::new(bind_to_free_socket().unwrap().0);
		let room_id = server.register_room(RoomTemplate::default()).unwrap();
		let rooms = server.get_rooms().unwrap();
		assert_eq!(rooms, vec![room_id]);
	}
}
