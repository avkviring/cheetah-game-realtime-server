use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};

use cheetah_matches_realtime_common::protocol::others::user_id::MemberAndRoomId;
use thiserror::Error;

use cheetah_matches_realtime_common::room::{RoomId, RoomMemberId};

use crate::debug::proto::admin;
use crate::debug::tracer::TracerSessionCommand;
use crate::room::forward::ForwardConfig;
use crate::room::template::config::{MemberTemplate, RoomTemplate};
use crate::room::RoomInfo;
use crate::server::manager::ManagementTask::TimeOffset;
use crate::server::rooms::RegisterUserError;
use crate::server::{DeleteMemberError, DeleteRoomError, RoomsServer};

///
/// Управление сервером
/// - запуск сервера в отдельном потоке
/// - связь с сервером через Sender
///
pub struct RoomsServerManager {
	handler: Option<JoinHandle<Result<(), io::Error>>>,
	sender: Sender<ManagementTask>,
	halt_signal: Arc<AtomicBool>,
	pub created_room_counter: usize,
}

pub enum ManagementTask {
	CreateRoom(RoomTemplate, Sender<RoomId>),
	CreateMember(RoomId, MemberTemplate, Sender<Result<RoomMemberId, RegisterUserError>>),
	DeleteMember(MemberAndRoomId, Sender<Result<(), DeleteMemberError>>),
	///
	/// Смещение текущего времени для тестирования
	///
	TimeOffset(Duration),
	Dump(RoomId, Sender<Result<admin::DumpResponse, String>>),
	GetRooms(Sender<Vec<RoomId>>),
	QueryRoom(RoomId, Sender<Option<RoomInfo>>),
	CommandTracerSessionTask(RoomId, TracerSessionCommand, Sender<Result<(), CommandTracerSessionTaskError>>),
	DeleteRoom(RoomId, Sender<Result<(), DeleteRoomError>>),
	PutForwardedCommandConfig(RoomId, ForwardConfig, Sender<Result<(), PutForwardedCommandConfigError>>),
}

#[derive(Debug, Error)]
pub enum CommandTracerSessionTaskError {
	#[error("RoomNotFound {0}")]
	RoomNotFound(RoomId),
	#[error("RecvTimeoutError")]
	RecvTimeoutError,
	#[error("ChannelSendError {0}")]
	ChannelSendError(String),
}

#[derive(Debug, Error)]
pub enum RegisterRoomRequestError {
	#[error("ChannelRecvError {0}")]
	ChannelRecvError(RecvTimeoutError),
	#[error("ChannelSendError {0}")]
	ChannelSendError(String),
}

#[derive(Debug, Error)]
pub enum CreateMemberRequestError {
	#[error("ChannelRecvError {0}")]
	ChannelRecvError(RecvTimeoutError),
	#[error("Error {0}")]
	Error(RegisterUserError),
	#[error("ChannelSendError {0}")]
	ChannelSendError(String),
}

#[derive(Debug, Error)]
pub enum RoomsServerManagerError {
	#[error("CannotCreateServerThread {0}")]
	CannotCreateServerThread(String),
}

#[derive(Debug, Error)]
pub enum DeleteRoomRequestError {
	#[error("ChannelRecvError {0}")]
	ChannelRecvError(RecvTimeoutError),
	#[error("ChannelSendError {0}")]
	ChannelSendError(String),
	#[error("DeleteRoomError {0}")]
	DeleteRoomError(DeleteRoomError),
}

#[derive(Debug, Error)]
pub enum DeleteMemberRequestError {
	#[error("ChannelRecvError {0}")]
	ChannelRecvError(RecvTimeoutError),
	#[error("ChannelSendError {0}")]
	ChannelSendError(String),
	#[error("DeleteMemberError {0}")]
	DeleteMemberError(DeleteMemberError),
}

#[derive(Debug, Error)]
pub enum PutForwardedCommandConfigError {
	#[error("RoomNotFound {0}")]
	RoomNotFound(RoomId),
	#[error("ChannelRecvError {0}")]
	ChannelRecvError(RecvTimeoutError),
	#[error("ChannelSendError {0}")]
	ChannelSendError(String),
}

impl Drop for RoomsServerManager {
	fn drop(&mut self) {
		self.halt_signal.store(true, Ordering::Relaxed);
	}
}

impl RoomsServerManager {
	pub fn new(socket: UdpSocket) -> Result<Self, RoomsServerManagerError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let halt_signal = Arc::new(AtomicBool::new(false));
		let cloned_halt_signal = halt_signal.clone();
		let handler = thread::Builder::new()
			.name(format!("server({:?})", socket.local_addr()))
			.spawn(move || match RoomsServer::new(socket, receiver, halt_signal) {
				Ok(server) => {
					server.run();
					Ok(())
				}
				Err(e) => {
					tracing::error!("Error running network thread {:?}", e);
					Err(e)
				}
			})
			.map_err(|e| RoomsServerManagerError::CannotCreateServerThread(format!("{:?}", e)))?;
		Ok(Self {
			handler: Some(handler),
			sender,
			halt_signal: cloned_halt_signal,
			created_room_counter: 0,
		})
	}

	pub fn get_rooms(&self) -> Result<Vec<RoomId>, String> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(ManagementTask::GetRooms(sender)).map_err(|e| format!("{:?}", e))?;
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(rooms) => Ok(rooms),
			Err(e) => Err(format!("{:?}", e)),
		}
	}

	pub fn query_room(&self, room_id: u64) -> Result<Option<RoomInfo>, String> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender
			.send(ManagementTask::QueryRoom(room_id, sender))
			.map_err(|e| format!("{:?}", e))?;
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(maybe_room_info) => Ok(maybe_room_info),
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
			.map_err(|e| CommandTracerSessionTaskError::ChannelSendError(format!("{:?}", e)))?;
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
			.map_err(|e| RegisterRoomRequestError::ChannelSendError(format!("{:?}", e)))?;
		self.created_room_counter += 1;
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(room_id) => {
				tracing::info!("[server] create room({:?})", room_id);
				Ok(room_id)
			}
			Err(e) => {
				tracing::error!("[server] fail create room");
				Err(RegisterRoomRequestError::ChannelRecvError(e))
			}
		}
	}

	/// закрыть соединение с пользователем и удалить его из комнаты
	pub fn delete_member(&mut self, id: MemberAndRoomId) -> Result<(), DeleteMemberRequestError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender
			.send(ManagementTask::DeleteMember(id, sender))
			.map_err(|e| DeleteMemberRequestError::ChannelSendError(format!("{:?}", e)))?;
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(Ok(_)) => {
				tracing::info!("[server] delete member({:?})", id);
				Ok(())
			}
			Ok(Err(e)) => {
				tracing::error!("[server] fail delete member({:?})", id);
				Err(DeleteMemberRequestError::DeleteMemberError(e))
			}
			Err(e) => {
				tracing::error!("[server] timeout delete member({:?})", id);
				Err(DeleteMemberRequestError::ChannelRecvError(e))
			}
		}
	}

	/// удалить комнату с севрера и закрыть соединение со всеми пользователями
	pub fn delete_room(&mut self, room_id: RoomId) -> Result<(), DeleteRoomRequestError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender
			.send(ManagementTask::DeleteRoom(room_id, sender))
			.map_err(|e| DeleteRoomRequestError::ChannelSendError(format!("{:?}", e)))?;
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(Ok(_)) => {
				self.created_room_counter -= 1;
				tracing::info!("[server] delete room({:?})", room_id);
				Ok(())
			}
			Ok(Err(e)) => {
				tracing::error!("[server] fail delete room({:?})", room_id);
				Err(DeleteRoomRequestError::DeleteRoomError(e))
			}
			Err(e) => {
				tracing::error!("[server] timeout delete room({:?})", room_id);
				Err(DeleteRoomRequestError::ChannelRecvError(e))
			}
		}
	}

	pub fn create_member(&mut self, room_id: RoomId, template: MemberTemplate) -> Result<RoomMemberId, CreateMemberRequestError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender
			.send(ManagementTask::CreateMember(room_id, template.clone(), sender))
			.map_err(|e| CreateMemberRequestError::ChannelSendError(format!("{:?}", e)))?;
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
				Err(CreateMemberRequestError::ChannelRecvError(e))
			}
		}
	}

	pub fn put_forwarded_command_config(&mut self, room_id: RoomId, config: ForwardConfig) -> Result<(), PutForwardedCommandConfigError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender
			.send(ManagementTask::PutForwardedCommandConfig(room_id, config, sender))
			.map_err(|e| PutForwardedCommandConfigError::ChannelSendError(format!("{:?}", e)))?;
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(Ok(_)) => {
				tracing::info!("[server] put forward command config({:?})", room_id);
				Ok(())
			}
			Ok(Err(e)) => {
				tracing::error!("[server] fail put forward command config({:?})", room_id);
				Err(e)
			}
			Err(e) => {
				tracing::error!("[server] timeout put forward command config({:?})", room_id);
				Err(PutForwardedCommandConfigError::ChannelRecvError(e))
			}
		}
	}

	pub fn set_time_offset(&self, duration: Duration) -> Result<(), String> {
		self.sender.send(TimeOffset(duration)).map_err(|e| format!("{:?}", e))
	}

	pub fn join(&mut self) -> Result<(), io::Error> {
		self.handler.take().unwrap().join().unwrap()
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

#[cfg(test)]
mod test {
	use cheetah_matches_realtime_common::network::bind_to_free_socket;

	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::server::manager::RoomsServerManager;

	#[test]
	fn should_increment_created_room_count() {
		let mut server = RoomsServerManager::new(bind_to_free_socket().unwrap()).unwrap();
		server.create_room(RoomTemplate::default()).unwrap();
		assert_eq!(server.created_room_counter, 1);
	}

	#[test]
	fn should_get_rooms() {
		let mut server = RoomsServerManager::new(bind_to_free_socket().unwrap()).unwrap();
		let room_id = server.create_room(RoomTemplate::default()).unwrap();
		let rooms = server.get_rooms().unwrap();
		assert_eq!(rooms, vec![room_id]);
	}

	#[test]
	fn should_create_member() {
		let mut server = RoomsServerManager::new(bind_to_free_socket().unwrap()).unwrap();
		let room_id = server.create_room(RoomTemplate::default()).unwrap();
		let member_id = server.create_member(room_id, MemberTemplate::default()).unwrap();

		assert_eq!(member_id, 1);
	}

	#[test]
	fn should_get_room_info() {
		let mut server = RoomsServerManager::new(bind_to_free_socket().unwrap()).unwrap();
		let room_id = server.create_room(RoomTemplate::default()).unwrap();
		for _ in 0..5 {
			server.create_member(room_id, MemberTemplate::default()).unwrap();
		}
		let room_info = server.query_room(room_id).unwrap().unwrap();

		assert_eq!(room_info.member_count, 5);
	}
}
