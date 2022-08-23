use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use cheetah_libraries_microservice::trace::Trace;
use cheetah_matches_realtime_common::room::RoomId;

use crate::grpc::proto::internal::realtime_server::Realtime;
use crate::grpc::proto::internal::*;
use crate::room::template::config::MemberTemplate;
use crate::server::manager::ServerManager;

mod from;
pub mod proto;

pub struct RealtimeInternalService {
	pub server_manager: Arc<Mutex<ServerManager>>,
}

impl RealtimeInternalService {
	pub fn new(server_manager: Arc<Mutex<ServerManager>>) -> Self {
		RealtimeInternalService { server_manager }
	}

	async fn register_user(
		&self,
		room_id: RoomId,
		template: MemberTemplate,
	) -> Result<Response<CreateMemberResponse>, Status> {
		let mut server = self.server_manager.lock().await;
		server
			.register_user(room_id, template.clone())
			.trace_err(format!("Register plugin user to room {}", room_id))
			.map_err(Status::internal)
			.map(|user_id| {
				Response::new(CreateMemberResponse {
					user_id: user_id as u32,
					private_key: template.private_key.to_vec(),
				})
			})
	}
}

#[tonic::async_trait]
impl Realtime for RealtimeInternalService {
	async fn create_room(
		&self,
		request: Request<RoomTemplate>,
	) -> Result<Response<RoomIdResponse>, Status> {
		let mut server = self.server_manager.lock().await;
		let template = crate::room::template::config::RoomTemplate::from(request.into_inner());
		let template_name = template.name.clone();
		server
			.register_room(template)
			.trace_err(format!("Create room with template {}", template_name))
			.map_err(Status::internal)
			.map(|id| Response::new(RoomIdResponse { room_id: id }))
	}

	async fn create_member(
		&self,
		request: Request<CreateMemberRequest>,
	) -> Result<Response<CreateMemberResponse>, Status> {
		let request = request.into_inner();
		self.register_user(
			request.room_id,
			crate::room::template::config::MemberTemplate::from(request.user.unwrap()),
		)
		.await
	}

	async fn create_super_member(
		&self,
		request: Request<CreateSuperMemberRequest>,
	) -> Result<Response<CreateMemberResponse>, Status> {
		let request = request.into_inner();
		self.register_user(request.room_id, MemberTemplate::new_super_member())
			.await
	}

	async fn probe(
		&self,
		_request: Request<ProbeRequest>,
	) -> Result<Response<ProbeResponse>, Status> {
		Ok(Response::new(ProbeResponse {}))
	}

	type WatchCreatedRoomEventStream = ReceiverStream<Result<RoomIdResponse, Status>>;

	async fn watch_created_room_event(
		&self,
		_request: Request<EmptyRequest>,
	) -> Result<Response<Self::WatchCreatedRoomEventStream>, Status> {
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
}

#[cfg(test)]
mod test {
	use std::sync::Arc;

	use tokio::sync::Mutex;
	use tokio_stream::wrappers::ReceiverStream;
	use tokio_stream::StreamExt;
	use tonic::{Request, Status};

	use cheetah_matches_realtime_common::network::bind_to_free_socket;

	use crate::grpc::proto::internal::realtime_server::Realtime;
	use crate::grpc::proto::internal::{EmptyRequest, RoomIdResponse};
	use crate::grpc::RealtimeInternalService;
	use crate::room::template::config::RoomTemplate;
	use crate::server::manager::ServerManager;

	#[tokio::test]
	async fn test_watch_created_room_event() {
		let server_manager = Arc::new(Mutex::new(ServerManager::new(
			bind_to_free_socket().unwrap().0,
		)));

		let first_room_id = server_manager
			.lock()
			.await
			.register_room(RoomTemplate::default())
			.unwrap();

		let service = RealtimeInternalService::new(server_manager.clone());
		let mut response: ReceiverStream<Result<RoomIdResponse, Status>> = service
			.watch_created_room_event(Request::new(EmptyRequest {}))
			.await
			.unwrap()
			.into_inner();

		let actual = response.try_next().await;
		assert_eq!(actual.unwrap().unwrap().room_id, first_room_id);

		let second_room_id = server_manager
			.lock()
			.await
			.register_room(RoomTemplate::default())
			.unwrap();

		let actual = response.try_next().await;
		assert_eq!(actual.unwrap().unwrap().room_id, second_room_id);
	}
}
