use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::{mpsc, MutexGuard};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use cheetah_matches_realtime_common::commands::field::FieldId;
use cheetah_matches_realtime_common::commands::CommandTypeId;
use cheetah_matches_realtime_common::constants::GameObjectTemplateId;
use cheetah_matches_realtime_common::protocol::others::user_id::MemberAndRoomId;
use cheetah_matches_realtime_common::room::RoomId;

use crate::grpc::proto::internal::realtime_server::Realtime;
use crate::grpc::proto::internal::*;
use crate::room::command::ServerCommandError;
use crate::room::forward::ForwardConfig;
use crate::room::template::config::MemberTemplate;
use crate::room::RoomInfo;
use crate::server::manager::{TaskError, TaskExecutionError};
use crate::RoomsServerManager;

mod from;
pub mod proto;

pub struct RealtimeInternalService {
	pub server_manager: Arc<Mutex<RoomsServerManager>>,
}

const SUPER_MEMBER_KEY_ENV: &str = "SUPER_MEMBER_KEY";

impl RealtimeInternalService {
	#[must_use]
	pub fn new(server_manager: Arc<Mutex<RoomsServerManager>>) -> Self {
		RealtimeInternalService { server_manager }
	}

	async fn register_user(&self, room_id: RoomId, template: MemberTemplate) -> Result<Response<CreateMemberResponse>, Status> {
		let private_key = template.private_key.clone();
		self.server_manager
			.lock()
			.await
			.create_member(room_id, template)
			.map_err(Status::from)
			.map(|user_id| {
				Response::new(CreateMemberResponse {
					user_id: u32::from(user_id),
					private_key: private_key.into(),
				})
			})
	}

	fn create_super_member_if_need(server: &mut MutexGuard<'_, RoomsServerManager>, room_id: RoomId) {
		if let Ok(key_from_env) = std::env::var(SUPER_MEMBER_KEY_ENV) {
			let key_from_env_bytes = key_from_env.as_bytes();
			let key = key_from_env_bytes.into();
			server.create_member(room_id, MemberTemplate::new_super_member_with_key(key)).unwrap();
		}
	}
}

#[tonic::async_trait]
impl Realtime for RealtimeInternalService {
	async fn create_room(&self, request: Request<RoomTemplate>) -> Result<Response<RoomIdResponse>, Status> {
		let mut server = self.server_manager.lock().await;
		let template = crate::room::template::config::RoomTemplate::from(request.into_inner());
		let room_id = server.create_room(template).map_err(Status::from)?;

		Self::create_super_member_if_need(&mut server, room_id);
		Ok(Response::new(RoomIdResponse { room_id }))
	}

	async fn create_member(&self, request: Request<CreateMemberRequest>) -> Result<Response<CreateMemberResponse>, Status> {
		let request = request.into_inner();
		self.register_user(
			request.room_id,
			crate::room::template::config::MemberTemplate::from(request.user.unwrap()),
		)
		.await
	}

	/// закрыть соединение с пользователем и удалить его из комнаты
	async fn delete_member(&self, request: Request<DeleteMemberRequest>) -> Result<Response<DeleteMemberResponse>, Status> {
		self.server_manager
			.lock()
			.await
			.delete_member(MemberAndRoomId {
				member_id: request
					.get_ref()
					.user_id
					.try_into()
					.map_err(|_| Status::invalid_argument("member_id is too big".to_string()))?,
				room_id: request.get_ref().room_id,
			})
			.map(|_| Response::new(DeleteMemberResponse {}))
			.map_err(Status::from)
	}

	async fn create_super_member(&self, request: Request<CreateSuperMemberRequest>) -> Result<Response<CreateMemberResponse>, Status> {
		let request = request.into_inner();
		self.register_user(request.room_id, MemberTemplate::new_super_member()).await
	}

	async fn probe(&self, _request: Request<ProbeRequest>) -> Result<Response<ProbeResponse>, Status> {
		Ok(Response::new(ProbeResponse {}))
	}

	type WatchCreatedRoomEventStream = ReceiverStream<Result<RoomIdResponse, Status>>;

	async fn watch_created_room_event(&self, _request: Request<EmptyRequest>) -> Result<Response<Self::WatchCreatedRoomEventStream>, Status> {
		let (tx, rx) = mpsc::channel(64);
		let server_manager = self.server_manager.clone();
		tokio::spawn(async move {
			let mut present_rooms = HashSet::new();
			loop {
				let server = server_manager.lock().await;
				let rooms = server.get_rooms().unwrap();
				drop(server);
				for room_id in rooms {
					if !present_rooms.contains(&room_id) {
						present_rooms.insert(room_id);
						tx.send(Ok(RoomIdResponse { room_id })).await.unwrap();
					}
				}
				tokio::task::yield_now().await;
			}
		});
		Ok(Response::new(ReceiverStream::new(rx)))
	}

	/// удалить комнату с севрера и закрыть соединение со всеми пользователями
	async fn delete_room(&self, request: Request<DeleteRoomRequest>) -> Result<Response<DeleteRoomResponse>, Status> {
		let room_id = request.get_ref().id as RoomId;
		let mut server = self.server_manager.lock().await;
		server
			.delete_room(room_id)
			.map(|_| Response::new(DeleteRoomResponse {}))
			.map_err(Status::from)
	}

	async fn put_forwarded_command_config(
		&self,
		request: Request<PutForwardedCommandConfigRequest>,
	) -> Result<Response<PutForwardedCommandConfigResponse>, Status> {
		let command_type_id = request.get_ref().command_type_id;
		let command_type_id: CommandTypeId = num::FromPrimitive::from_u32(command_type_id)
			.ok_or_else(|| Status::invalid_argument(format!("unknown command_type_id {:?}", command_type_id)))?;

		let field_id = request.get_ref().field_id;
		let field_id = if let Some(field_id) = field_id {
			Some(FieldId::try_from(field_id).map_err(|e| Status::invalid_argument(format!("field_id is too large {:?} {:?}", field_id, e)))?)
		} else {
			None
		};

		let object_template_id = request.get_ref().template_id;
		let object_template_id = if let Some(object_template_id) = object_template_id {
			Some(
				GameObjectTemplateId::try_from(object_template_id)
					.map_err(|e| Status::invalid_argument(format!("object_template_id is too large {:?} {:?}", object_template_id, e)))?,
			)
		} else {
			None
		};

		self.server_manager
			.lock()
			.await
			.put_forwarded_command_config(
				request.get_ref().room_id as RoomId,
				ForwardConfig {
					command_type_id,
					field_id,
					object_template_id,
				},
			)
			.map(|_| Response::new(PutForwardedCommandConfigResponse {}))
			.map_err(Status::from)
	}

	async fn mark_room_as_ready(&self, request: Request<MarkRoomAsReadyRequest>) -> Result<Response<MarkRoomAsReadyResponse>, Status> {
		self.server_manager
			.lock()
			.await
			.mark_room_as_ready(request.get_ref().room_id as RoomId, request.get_ref().plugin_name.clone())
			.map(|_| Response::new(MarkRoomAsReadyResponse {}))
			.map_err(Status::from)
	}

	async fn get_room_info(&self, request: Request<GetRoomInfoRequest>) -> Result<Response<GetRoomInfoResponse>, Status> {
		let room_id = request.get_ref().room_id as RoomId;

		self.server_manager
			.lock()
			.await
			.get_room_info(room_id)
			.map(|room_info| Response::new(GetRoomInfoResponse::from(room_info)))
			.map_err(Status::from)
	}
}

impl From<TaskError> for Status {
	fn from(e: TaskError) -> Self {
		match e {
			TaskError::ChannelRecvError(e) => Status::deadline_exceeded(e.to_string()),
			TaskError::ChannelSendError(e) => Status::unavailable(e.to_string()),
			TaskError::UnexpectedResultError => Status::internal("unexpected management task result type"),
			TaskError::TaskExecutionError(TaskExecutionError::RoomNotFound(e)) => Status::not_found(e.to_string()),
			TaskError::TaskExecutionError(TaskExecutionError::UnknownPluginName(e)) => Status::invalid_argument(e),
			TaskError::TaskExecutionError(TaskExecutionError::ServerCommandError(e)) => match e {
				ServerCommandError::MemberNotFound(e) => Status::not_found(e.to_string()),
				ServerCommandError::RoomNotFound(e) => Status::not_found(e.to_string()),
				e => Status::internal(e.to_string()),
			},
		}
	}
}

impl From<RoomInfo> for GetRoomInfoResponse {
	fn from(room_info: RoomInfo) -> Self {
		Self {
			room_id: room_info.room_id,
			ready: room_info.ready,
		}
	}
}

#[cfg(test)]
mod test {
	use fnv::FnvHashSet;
	use std::sync::Arc;

	use tokio::sync::Mutex;
	use tokio_stream::wrappers::ReceiverStream;
	use tokio_stream::StreamExt;
	use tonic::{Code, Request, Status};

	use cheetah_matches_realtime_common::commands::CommandTypeId;
	use cheetah_matches_realtime_common::network::bind_to_free_socket;

	use crate::grpc::proto::internal::realtime_server::Realtime;
	use crate::grpc::proto::internal::{
		DeleteMemberRequest, DeleteRoomRequest, EmptyRequest, GetRoomInfoRequest, MarkRoomAsReadyRequest, PutForwardedCommandConfigRequest,
		RoomIdResponse,
	};
	use crate::grpc::{RealtimeInternalService, SUPER_MEMBER_KEY_ENV};
	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::server::manager::RoomsServerManager;

	#[tokio::test]
	async fn test_watch_created_room_event() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));

		let first_room_id = server_manager.lock().await.create_room(RoomTemplate::default()).unwrap();

		let service = RealtimeInternalService::new(server_manager.clone());
		let mut response: ReceiverStream<Result<RoomIdResponse, Status>> = service
			.watch_created_room_event(Request::new(EmptyRequest {}))
			.await
			.unwrap()
			.into_inner();

		let actual = response.try_next().await;
		assert_eq!(actual.unwrap().unwrap().room_id, first_room_id);

		let second_room_id = server_manager.lock().await.create_room(RoomTemplate::default()).unwrap();

		let actual = response.try_next().await;
		assert_eq!(actual.unwrap().unwrap().room_id, second_room_id);
	}

	#[tokio::test]
	async fn test_create_super_member() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));

		std::env::set_var(SUPER_MEMBER_KEY_ENV, "some-key");
		let service = RealtimeInternalService::new(server_manager.clone());
		let room_id = service.create_room(Request::new(Default::default())).await.unwrap().into_inner();

		let dump_response = server_manager.lock().await.dump(room_id.room_id).unwrap();
		assert!(!dump_response.users.is_empty());
	}

	#[tokio::test]
	async fn test_delete_room() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));

		let service = RealtimeInternalService::new(server_manager.clone());
		let room_id = service.create_room(Request::new(Default::default())).await.unwrap().into_inner().room_id;

		assert!(service.delete_room(Request::new(DeleteRoomRequest { id: room_id })).await.is_ok());
	}

	#[tokio::test]
	async fn test_delete_room_not_exist() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));

		let service = RealtimeInternalService::new(server_manager.clone());

		assert!(service.delete_room(Request::new(Default::default())).await.is_err());
	}

	#[tokio::test]
	async fn test_delete_member() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeInternalService::new(server_manager.clone());

		let room_id = service.create_room(Request::new(Default::default())).await.unwrap().into_inner().room_id;
		let user_id = service
			.register_user(room_id, MemberTemplate::default())
			.await
			.unwrap()
			.into_inner()
			.user_id;
		assert!(
			!server_manager.lock().await.dump(room_id).unwrap().users.is_empty(),
			"room should not be empty"
		);

		assert!(
			service
				.delete_member(Request::new(DeleteMemberRequest { room_id, user_id }))
				.await
				.is_ok(),
			"delete_member should return ok"
		);

		println!(
			"user_id={:?} dump={:?}",
			user_id,
			server_manager.lock().await.dump(room_id).unwrap().users
		);
		assert!(
			!server_manager.lock().await.dump(room_id).unwrap().users.iter().any(|u| u.id == user_id),
			"deleted member should not be in the room"
		);
	}

	#[tokio::test]
	async fn test_delete_member_room_not_exist() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeInternalService::new(server_manager.clone());

		let res = service.delete_member(Request::new(DeleteMemberRequest { user_id: 0, room_id: 0 })).await;

		assert!(matches!(res.unwrap_err().code(), Code::NotFound), "delete_member should return not_found");
	}

	#[tokio::test]
	async fn test_put_forwarded_command_config() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeInternalService::new(server_manager.clone());

		let room_id = service.create_room(Request::new(Default::default())).await.unwrap().into_inner().room_id;

		assert!(
			service
				.put_forwarded_command_config(Request::new(PutForwardedCommandConfigRequest {
					room_id,
					command_type_id: CommandTypeId::AttachToRoom as _,
					field_id: None,
					template_id: None
				}))
				.await
				.is_ok(),
			"put_forwarded_command_config should return ok"
		);
		assert!(
			service
				.put_forwarded_command_config(Request::new(PutForwardedCommandConfigRequest {
					room_id,
					command_type_id: CommandTypeId::AttachToRoom as _,
					field_id: None,
					template_id: None
				}))
				.await
				.is_ok(),
			"put_forwarded_command_config should be idempotent"
		);
	}

	#[tokio::test]
	async fn test_put_forwarded_command_config_room_not_found() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeInternalService::new(server_manager.clone());

		let res = service
			.put_forwarded_command_config(Request::new(PutForwardedCommandConfigRequest {
				room_id: 0,
				command_type_id: CommandTypeId::AttachToRoom as _,
				field_id: None,
				template_id: None,
			}))
			.await;

		assert!(
			matches!(res.unwrap_err().code(), Code::NotFound),
			"put_forwarded_command_config should return not_found"
		);
	}

	#[tokio::test]
	async fn test_put_forwarded_command_config_invalid_argument() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeInternalService::new(server_manager.clone());

		let room_id = service.create_room(Request::new(Default::default())).await.unwrap().into_inner().room_id;

		let tests = [(30, 0, 0), (0, u32::from(u16::MAX) + 1, 0), (0, 0, u32::from(u16::MAX) + 1)];
		for (command_type_id, field_id, template_id) in tests {
			let res = service
				.put_forwarded_command_config(Request::new(PutForwardedCommandConfigRequest {
					room_id,
					command_type_id,
					field_id: Some(field_id),
					template_id: Some(template_id),
				}))
				.await;

			assert!(
				matches!(res.unwrap_err().code(), Code::InvalidArgument),
				"put_forwarded_command_config should return invalid_Argument"
			);
		}
	}

	#[tokio::test]
	async fn test_mark_room_as_ready() {
		let plugin_name = "plugin_1";
		let plugin_names = FnvHashSet::from_iter([plugin_name.to_string()]);
		let server_manager = Arc::new(Mutex::new(RoomsServerManager::new(bind_to_free_socket().unwrap(), plugin_names).unwrap()));
		let service = RealtimeInternalService::new(server_manager.clone());
		let room_id = service.create_room(Request::new(Default::default())).await.unwrap().into_inner().room_id;

		let ready = service
			.get_room_info(Request::new(GetRoomInfoRequest { room_id }))
			.await
			.unwrap()
			.into_inner()
			.ready;
		assert!(!ready, "room should not be ready after creation if plugins list is configured");

		assert!(
			service
				.mark_room_as_ready(Request::new(MarkRoomAsReadyRequest {
					room_id,
					plugin_name: plugin_name.to_string(),
				}))
				.await
				.is_ok(),
			"mark_room_as_ready should return ok"
		);

		let ready = service
			.get_room_info(Request::new(GetRoomInfoRequest { room_id }))
			.await
			.unwrap()
			.into_inner()
			.ready;
		assert!(ready, "room should be ready after all plugins have called mark_room_as_ready");

		assert!(
			service
				.mark_room_as_ready(Request::new(MarkRoomAsReadyRequest {
					room_id,
					plugin_name: plugin_name.to_string(),
				}))
				.await
				.is_ok(),
			"mark_room_as_ready should return after retries"
		);
	}

	#[tokio::test]
	async fn test_get_room_info_not_found() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeInternalService::new(server_manager.clone());
		let res = service.get_room_info(Request::new(GetRoomInfoRequest { room_id: 0 })).await;

		assert!(matches!(res.unwrap_err().code(), Code::NotFound), "get_room_info should return not_found");
	}

	#[tokio::test]
	async fn test_mark_room_as_ready_room_not_found() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeInternalService::new(server_manager.clone());
		let res = service
			.mark_room_as_ready(Request::new(MarkRoomAsReadyRequest {
				room_id: 0,
				plugin_name: "plugin_1".to_string(),
			}))
			.await;

		assert!(
			matches!(res.unwrap_err().code(), Code::NotFound),
			"mark_room_as_ready should return not_found"
		);
	}

	#[tokio::test]
	async fn test_mark_room_as_ready_unknown_plugin_name() {
		let server_manager = Arc::new(Mutex::new(new_server_manager()));
		let service = RealtimeInternalService::new(server_manager.clone());
		let room_id = service.create_room(Request::new(Default::default())).await.unwrap().into_inner().room_id;

		let res = service
			.mark_room_as_ready(Request::new(MarkRoomAsReadyRequest {
				room_id,
				plugin_name: "unknown_plugin_name".to_string(),
			}))
			.await;

		assert!(
			matches!(res.unwrap_err().code(), Code::InvalidArgument),
			"mark_room_as_ready should return invalid_argument"
		);
	}

	fn new_server_manager() -> RoomsServerManager {
		RoomsServerManager::new(bind_to_free_socket().unwrap(), FnvHashSet::default()).unwrap()
	}
}
