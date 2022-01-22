use std::sync::{Arc, Mutex};

use cheetah_microservice::tonic::{Request, Response};

use crate::debug::proto::admin;
use crate::server::manager::ServerManager;

pub struct RelayAdminGRPCService {
	pub manager: Arc<Mutex<ServerManager>>,
}
impl RelayAdminGRPCService {
	pub fn new(manager: Arc<Mutex<ServerManager>>) -> Self {
		Self { manager }
	}
}

#[tonic::async_trait]
impl admin::relay_server::Relay for RelayAdminGRPCService {
	async fn get_rooms(
		&self,
		_request: Request<admin::GetRoomsRequest>,
	) -> Result<Response<admin::GetRoomsResponse>, tonic::Status> {
		let manager = self.manager.lock().unwrap();
		match manager.get_rooms() {
			Ok(rooms) => Result::Ok(Response::new(admin::GetRoomsResponse { rooms })),
			Err(e) => Result::Err(tonic::Status::internal(e)),
		}
	}
}
