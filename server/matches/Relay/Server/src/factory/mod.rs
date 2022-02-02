use std::sync::{Arc, Mutex};

use tonic::{Request, Response, Status};

use crate::factory::proto::internal::*;
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
}

#[tonic::async_trait]
impl crate::factory::proto::internal::relay_server::Relay for RelayGRPCService {
	async fn create_room(&self, request: tonic::Request<RoomTemplate>) -> Result<Response<CreateRoomResponse>, Status> {
		let mut server = self.relay_server.lock().unwrap();
		let template = crate::room::template::config::RoomTemplate::from(request.into_inner());
		match server.register_room(template) {
			Ok(id) => Result::Ok(Response::new(CreateRoomResponse { id })),
			Err(e) => Result::Err(Status::not_found(format!("{:?}", e))),
		}
	}

	async fn attach_user(&self, request: Request<AttachUserRequest>) -> Result<Response<AttachUserResponse>, Status> {
		let mut server = self.relay_server.lock().unwrap();
		let request = request.into_inner();
		let template = crate::room::template::config::MemberTemplate::from(request.user.unwrap());
		let private_key = template.private_key;
		match server.register_user(request.room_id, template) {
			Ok(user_id) => Result::Ok(Response::new(AttachUserResponse {
				user_id: user_id as u32,
				private_key: private_key.to_vec(),
			})),
			Err(e) => Result::Err(Status::internal(format!("{:?}", e))),
		}
	}
}
