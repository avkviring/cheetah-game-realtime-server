use std::sync::{Arc, Mutex};

use tonic::{Request, Response, Status};

use cheetah_libraries_microservice::trace::ResultErrorTracer;

use crate::grpc::proto::internal::*;
use crate::server::manager::ServerManager;

mod from;
pub mod proto;

///
///
///
pub struct RelayGRPCService {
	pub relay_server: Arc<Mutex<ServerManager>>,
}

impl RelayGRPCService {
	pub fn new(relay_server: Arc<Mutex<ServerManager>>) -> Self {
		RelayGRPCService { relay_server }
	}
}

#[tonic::async_trait]
impl relay_server::Relay for RelayGRPCService {
	async fn create_room(
		&self,
		request: Request<RoomTemplate>,
	) -> Result<Response<CreateRoomResponse>, Status> {
		let mut server = self.relay_server.lock().unwrap();
		let template = crate::room::template::config::RoomTemplate::from(request.into_inner());
		let template_name = template.name.clone();
		server
			.register_room(template)
			.trace_and_map_err(
				format!("Create room with template {}", template_name),
				Status::internal,
			)
			.map(|id| Response::new(CreateRoomResponse { id }))
	}

	async fn attach_user(
		&self,
		request: Request<AttachUserRequest>,
	) -> Result<Response<AttachUserResponse>, Status> {
		let mut server = self.relay_server.lock().unwrap();
		let request = request.into_inner();
		let template = crate::room::template::config::MemberTemplate::from(request.user.unwrap());
		let private_key = template.private_key;
		server
			.register_user(request.room_id, template)
			.trace_and_map_err(
				format!("Attach user to room {}", request.room_id),
				Status::internal,
			)
			.map(|user_id| {
				Response::new(AttachUserResponse {
					user_id: user_id as u32,
					private_key: private_key.to_vec(),
				})
			})
	}

	async fn probe(
		&self,
		_request: Request<ProbeRequest>,
	) -> Result<Response<ProbeResponse>, Status> {
		Ok(Response::new(ProbeResponse {}))
	}
}
