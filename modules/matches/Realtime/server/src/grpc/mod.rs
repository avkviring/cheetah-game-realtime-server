use std::sync::{Arc, Mutex};

use tonic::{Request, Response, Status};

use cheetah_libraries_microservice::trace::Trace;
use cheetah_matches_realtime_common::room::RoomId;

use crate::grpc::proto::internal::realtime_server::Realtime;
use crate::grpc::proto::internal::*;
use crate::room::template::config::MemberTemplate;
use crate::server::manager::ServerManager;

mod from;
pub mod proto;

pub struct RelayGRPCService {
	pub relay_server: Arc<Mutex<ServerManager>>,
}

impl RelayGRPCService {
	pub fn new(relay_server: Arc<Mutex<ServerManager>>) -> Self {
		RelayGRPCService { relay_server }
	}

	fn register_user(
		&self,
		room_id: RoomId,
		template: MemberTemplate,
	) -> Result<Response<CreateMemberResponse>, Status> {
		let mut server = self.relay_server.lock().unwrap();
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
impl Realtime for RelayGRPCService {
	async fn create_room(
		&self,
		request: Request<RoomTemplate>,
	) -> Result<Response<CreateRoomResponse>, Status> {
		let mut server = self.relay_server.lock().unwrap();
		let template = crate::room::template::config::RoomTemplate::from(request.into_inner());
		let template_name = template.name.clone();
		server
			.register_room(template)
			.trace_err(format!("Create room with template {}", template_name))
			.map_err(Status::internal)
			.map(|id| Response::new(CreateRoomResponse { id }))
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
	}

	async fn create_super_member(
		&self,
		request: Request<CreateSuperMemberRequest>,
	) -> Result<Response<CreateMemberResponse>, Status> {
		let request = request.into_inner();
		self.register_user(request.room_id, MemberTemplate::new_super_member())
	}

	async fn probe(
		&self,
		_request: Request<ProbeRequest>,
	) -> Result<Response<ProbeResponse>, Status> {
		Ok(Response::new(ProbeResponse {}))
	}
}
