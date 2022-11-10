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

use thiserror::Error;

use admin::DumpResponse;
use cheetah_matches_realtime_common::protocol::others::user_id::MemberAndRoomId;
use cheetah_matches_realtime_common::room::RoomId;

use crate::debug::proto::admin;
use crate::room::command::ServerCommandError;
use crate::server::manager::ManagementTask::TimeOffset;
use crate::server::manager::{CommandTracerSessionTaskError, ManagementTask, MarkRoomAsReadyError, PutForwardedCommandConfigError};
use crate::server::measurers::Measurers;
use crate::server::network::NetworkLayer;
use crate::server::rooms::{RoomNotFoundError, Rooms};

pub mod manager;
pub mod measurers;
pub mod network;
pub mod rooms;

#[derive(Debug, Error)]
pub enum DeleteRoomError {
	#[error("RoomNotFound")]
	RoomNotFound(RoomNotFoundError),
}

#[derive(Debug, Error)]
pub enum DeleteMemberError {
	#[error("ServerCommandError {0}")]
	ServerCommand(ServerCommandError),
}

///
/// Собственно сетевой сервер, запускается в отдельном потоке, обрабатывает сетевые команды,
/// поддерживает одновременно несколько комнат
///
pub struct RoomsServer {
	network_layer: NetworkLayer,
	rooms: Rooms,
	receiver: Receiver<ManagementTask>,
	halt_signal: Arc<AtomicBool>,
	time_offset: Option<Duration>,
	measurers: Rc<RefCell<Measurers>>,
	plugin_names: FnvHashSet<String>,
}

impl RoomsServer {
	pub fn new(
		socket: UdpSocket,
		receiver: Receiver<ManagementTask>,
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
		while let Ok(request) = self.receiver.try_recv() {
			match request {
				ManagementTask::CreateRoom(template, sender) => {
					let result = self.rooms.create_room(template.clone());
					match sender.send(result) {
						Ok(_) => {}
						Err(e) => {
							tracing::error!("[Request::RegisterRoom] error send response {:?}", e);
						}
					}
				}
				ManagementTask::DeleteRoom(room_id, sender) => {
					let result = self.delete_room(room_id);
					match sender.send(result) {
						Ok(_) => {}
						Err(e) => {
							tracing::error!("[Request::DeleteRoom] error send response {:?}", e);
						}
					}
				}
				ManagementTask::CreateMember(room_id, user_template, sender) => {
					let result = self.rooms.register_user(room_id, user_template.clone());
					if let Ok(user_id) = &result {
						self.network_layer.register_user(now, room_id, *user_id, user_template);
					}
					if let Err(e) = sender.send(result) {
						tracing::error!("[Request::RegisterUser] error send response {:?}", e);
					}
				}
				ManagementTask::DeleteMember(id, sender) => {
					let result = self.delete_member(id);
					match sender.send(result) {
						Ok(_) => {}
						Err(e) => {
							tracing::error!("[Request::DeleteMember] error send response {:?}", e);
						}
					}
				}
				TimeOffset(time_offset) => {
					self.time_offset = Some(time_offset);
				}
				ManagementTask::Dump(room_id, sender) => match self.rooms.room_by_id.get(&room_id) {
					None => {
						if let Err(e) = sender.send(Err(format!("Room not found {:?}", room_id).to_string())) {
							tracing::error!("[Request::Dump] error send response {:?}", e);
						}
					}
					Some(room) => {
						let response: DumpResponse = DumpResponse::from(room);
						let result = Ok(response);
						if let Err(e) = sender.send(result) {
							tracing::error!("[Request::Dump] error send response {:?}", e);
						}
					}
				},
				ManagementTask::GetRooms(sender) => match sender.send(self.rooms.room_by_id.keys().copied().collect()) {
					Ok(_) => {}
					Err(e) => {
						tracing::error!("[Request::GetRooms] error send response {:?}", e);
					}
				},
				ManagementTask::CommandTracerSessionTask(room_id, task, sender) => match self.rooms.room_by_id.get_mut(&room_id) {
					None => {
						if let Err(e) = sender.send(Err(CommandTracerSessionTaskError::RoomNotFound(room_id))) {
							tracing::error!("[Request::CommandTracerSessionTask] error send response {:?}", e);
						}
					}
					Some(room) => {
						room.command_trace_session.clone().borrow_mut().execute_task(task);
						if let Err(e) = sender.send(Ok(())) {
							tracing::error!("[Request::CommandTracerSessionTask] error send response {:?}", e);
						}
					}
				},
				ManagementTask::PutForwardedCommandConfig(room_id, config, sender) => {
					let result = if let Some(room) = self.rooms.room_by_id.get_mut(&room_id) {
						room.put_forwarded_command_config(config);
						Ok(())
					} else {
						Err(PutForwardedCommandConfigError::RoomNotFound(room_id))
					};
					match sender.send(result) {
						Ok(_) => {}
						Err(e) => {
							tracing::error!("[Request::PutForwardedCommandConfig] error send response {:?}", e);
						}
					}
				}
				ManagementTask::MarkRoomAsReady(room_id, plugin_name, sender) => {
					let result = if let Some(room) = self.rooms.room_by_id.get_mut(&room_id) {
						if self.plugin_names.contains(&plugin_name) {
							room.mark_room_as_ready(&plugin_name);
							Ok(())
						} else {
							Err(MarkRoomAsReadyError::UnknownPluginName(plugin_name))
						}
					} else {
						Err(MarkRoomAsReadyError::RoomNotFound(room_id))
					};
					match sender.send(result) {
						Ok(_) => {}
						Err(e) => {
							tracing::error!("[Request::MarkRoomAsReady] error send response {:?}", e);
						}
					}
				}
				ManagementTask::GetRoomInfo(room_id, sender) => {
					let result = if let Some(room) = self.rooms.room_by_id.get(&room_id) {
						Ok(room.get_info())
					} else {
						Err(manager::RoomNotFoundError::RoomNotFound(room_id))
					};
					match sender.send(result) {
						Ok(_) => {}
						Err(e) => {
							tracing::error!("[Request::GetRoomInfo] error send response {:?}", e);
						}
					}
				}
			}
		}
	}

	/// удалить комнату с севрера и закрыть соединение со всеми пользователями
	fn delete_room(&mut self, room_id: RoomId) -> Result<(), DeleteRoomError> {
		let room = self.rooms.take_room(&room_id).map_err(DeleteRoomError::RoomNotFound)?;
		let ids = room.members.into_keys().map(|member_id| MemberAndRoomId { member_id, room_id });
		self.network_layer.disconnect_users(ids);
		Ok(())
	}

	/// закрыть соединение с пользователем и удалить его из комнаты
	fn delete_member(&mut self, id: MemberAndRoomId) -> Result<(), DeleteMemberError> {
		self.network_layer.disconnect_users([id].into_iter());
		self.rooms.user_disconnected(&id).map_err(DeleteMemberError::ServerCommand)
	}
}
