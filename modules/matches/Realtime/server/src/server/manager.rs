use fnv::FnvHashSet;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, SendError, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use thiserror::Error;

use cheetah_matches_realtime_common::protocol::others::member_id::MemberAndRoomId;
use cheetah_matches_realtime_common::room::{RoomId, RoomMemberId};

use crate::debug::proto::admin;
use crate::debug::tracer::TracerSessionCommand;
use crate::room::command::ServerCommandError;
use crate::room::forward::ForwardConfig;
use crate::room::template::config::{MemberTemplate, Permissions, RoomTemplate};
use crate::room::RoomInfo;
use crate::server::rooms::RoomNotFoundError;
use crate::server::RoomsServer;

///
/// Управление сервером
/// - запуск сервера в отдельном потоке
/// - связь с сервером через Sender
///
pub struct RoomsServerManager {
	sender: Sender<ChannelTask>,
	halt_signal: Arc<AtomicBool>,
	pub created_room_counter: usize,
}

#[derive(Debug)]
pub enum ManagementTask {
	CreateRoom(RoomTemplate),
	CreateMember(RoomId, MemberTemplate),
	DeleteMember(MemberAndRoomId),
	Dump(RoomId),
	GetRooms,
	CommandTracerSessionTask(RoomId, TracerSessionCommand),
	DeleteRoom(RoomId),
	PutForwardedCommandConfig(RoomId, ForwardConfig),
	MarkRoomAsReady(RoomId, String),
	GetRoomInfo(RoomId),
	UpdateRoomPermissions(RoomId, Permissions),
}

#[derive(Debug)]
pub enum ManagementTaskResult {
	CreateRoom(RoomId),
	CreateMember(RoomMemberId),
	DeleteMember,
	Dump(admin::DumpResponse),
	GetRooms(Vec<RoomId>),
	CommandTracerSessionTask,
	DeleteRoom,
	PutForwardedCommandConfig,
	MarkRoomAsReady,
	GetRoomInfo(RoomInfo),
	UpdateRoomPermissions,
}

#[derive(Error, Debug)]
pub enum RoomsServerManagerError {
	#[error("CannotCreateServerThread {0}")]
	CannotCreateServerThread(String),
}

#[derive(Error, Debug)]
pub enum TaskError {
	#[error("ChannelSendError {0}")]
	ChannelSendError(SendError<ChannelTask>),
	#[error("ChannelRecvError {0}")]
	ChannelRecvError(RecvTimeoutError),
	#[error("TaskExecutionError {0}")]
	TaskExecutionError(TaskExecutionError),
	#[error("UnexpectedResultError")]
	UnexpectedResultError,
}

#[derive(Error, Debug)]
pub enum TaskExecutionError {
	#[error("RoomNotFound {0}")]
	RoomNotFound(#[from] RoomNotFoundError),
	#[error("UnknownPluginName {0}")]
	UnknownPluginName(String),
	#[error("ServerCommandError {0}")]
	ServerCommandError(#[from] ServerCommandError),
}

pub struct ChannelTask {
	pub task: ManagementTask,
	pub sender: Sender<Result<ManagementTaskResult, TaskExecutionError>>,
}

impl Drop for RoomsServerManager {
	fn drop(&mut self) {
		self.halt_signal.store(true, Ordering::Relaxed);
	}
}

impl RoomsServerManager {
	pub fn new(socket: UdpSocket, plugin_names: FnvHashSet<String>) -> Result<Self, RoomsServerManagerError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let halt_signal = Arc::new(AtomicBool::new(false));
		let cloned_halt_signal = Arc::clone(&halt_signal);
		thread::Builder::new()
			.name(format!("server({:?})", socket.local_addr()))
			.spawn(move || match RoomsServer::new(socket, receiver, halt_signal, plugin_names) {
				Ok(server) => {
					server.run();
					Ok(())
				}
				Err(e) => {
					tracing::error!("Error running network thread {:?}", e);
					Err(e)
				}
			})
			.map_err(|e| RoomsServerManagerError::CannotCreateServerThread(format!("{e:?}")))?;
		Ok(Self {
			sender,
			halt_signal: cloned_halt_signal,
			created_room_counter: 0,
		})
	}

	pub(crate) fn get_rooms(&self) -> Result<Vec<RoomId>, TaskError> {
		self.execute_task(ManagementTask::GetRooms).map(|res| {
			if let ManagementTaskResult::GetRooms(rooms) = res {
				Ok(rooms)
			} else {
				Err(TaskError::UnexpectedResultError)
			}
		})?
	}

	pub fn create_room(&mut self, template: RoomTemplate) -> Result<RoomId, TaskError> {
		self.execute_task(ManagementTask::CreateRoom(template)).map(|res| {
			if let ManagementTaskResult::CreateRoom(room_id) = res {
				self.created_room_counter += 1;
				Ok(room_id)
			} else {
				Err(TaskError::UnexpectedResultError)
			}
		})?
	}

	/// закрыть соединение с пользователем и удалить его из комнаты
	pub fn delete_member(&mut self, id: MemberAndRoomId) -> Result<(), TaskError> {
		self.execute_task(ManagementTask::DeleteMember(id)).map(|_| ())
	}

	/// удалить комнату с сервера и закрыть соединение со всеми пользователями
	pub fn delete_room(&mut self, room_id: RoomId) -> Result<(), TaskError> {
		self.execute_task(ManagementTask::DeleteRoom(room_id)).map(|_| ())
	}

	pub fn create_member(&mut self, room_id: RoomId, template: MemberTemplate) -> Result<RoomMemberId, TaskError> {
		self.execute_task(ManagementTask::CreateMember(room_id, template)).map(|res| {
			if let ManagementTaskResult::CreateMember(id) = res {
				Ok(id)
			} else {
				Err(TaskError::UnexpectedResultError)
			}
		})?
	}

	pub(crate) fn put_forwarded_command_config(&mut self, room_id: RoomId, config: ForwardConfig) -> Result<(), TaskError> {
		self.execute_task(ManagementTask::PutForwardedCommandConfig(room_id, config)).map(|_| ())
	}

	pub(crate) fn mark_room_as_ready(&mut self, room_id: RoomId, plugin_name: String) -> Result<(), TaskError> {
		self.execute_task(ManagementTask::MarkRoomAsReady(room_id, plugin_name)).map(|_| ())
	}

	pub(crate) fn update_room_permissions(&mut self, room_id: RoomId, permissions: Permissions) -> Result<(), TaskError> {
		self.execute_task(ManagementTask::UpdateRoomPermissions(room_id, permissions)).map(|_| ())
	}

	pub(crate) fn get_room_info(&mut self, room_id: RoomId) -> Result<RoomInfo, TaskError> {
		self.execute_task(ManagementTask::GetRoomInfo(room_id)).map(|res| {
			if let ManagementTaskResult::GetRoomInfo(room_info) = res {
				Ok(room_info)
			} else {
				Err(TaskError::UnexpectedResultError)
			}
		})?
	}

	pub(crate) fn dump(&self, room_id: u64) -> Result<admin::DumpResponse, TaskError> {
		self.execute_task(ManagementTask::Dump(room_id)).map(|res| {
			if let ManagementTaskResult::Dump(resp) = res {
				Ok(resp)
			} else {
				Err(TaskError::UnexpectedResultError)
			}
		})?
	}

	///
	/// Выполнить задачу в `CommandTracerSessions` конкретной комнаты
	/// Подход с вложенным enum для отдельного класса задач применяется для изолирования функционала
	///
	pub(crate) fn execute_command_trace_sessions_task(&self, room_id: RoomId, task: TracerSessionCommand) -> Result<(), TaskError> {
		self.execute_task(ManagementTask::CommandTracerSessionTask(room_id, task)).map(|_| ())
	}

	fn execute_task(&self, task: ManagementTask) -> Result<ManagementTaskResult, TaskError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(ChannelTask { task, sender }).map_err(TaskError::ChannelSendError)?;
		match receiver.recv_timeout(Duration::from_secs(1)) {
			Ok(Ok(result)) => Ok(result),
			Ok(Err(e)) => Err(TaskError::TaskExecutionError(e)),
			Err(e) => Err(TaskError::ChannelRecvError(e)),
		}
	}

	pub(crate) fn get_halt_signal(&self) -> Arc<AtomicBool> {
		Arc::clone(&self.halt_signal)
	}

	pub fn shutdown(&mut self) {
		self.halt_signal.store(true, Ordering::Relaxed);
	}
}

#[cfg(test)]
mod test {
	use cheetah_matches_realtime_common::network::bind_to_free_socket;
	use fnv::FnvHashSet;

	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::server::manager::RoomsServerManager;

	#[test]
	fn should_increment_created_room_count() {
		let mut server = new_server_manager();
		server.create_room(RoomTemplate::default()).unwrap();
		assert_eq!(server.created_room_counter, 1);
	}

	#[test]
	fn should_get_rooms() {
		let mut server = new_server_manager();
		let room_id = server.create_room(RoomTemplate::default()).unwrap();
		let rooms = server.get_rooms().unwrap();
		assert_eq!(rooms, vec![room_id]);
	}

	#[test]
	fn should_create_member() {
		let mut server = new_server_manager();
		let room_id = server.create_room(RoomTemplate::default()).unwrap();
		let member_id = server.create_member(room_id, MemberTemplate::default()).unwrap();

		assert_eq!(member_id, 1);
	}

	fn new_server_manager() -> RoomsServerManager {
		RoomsServerManager::new(bind_to_free_socket().unwrap(), FnvHashSet::default()).unwrap()
	}
}
