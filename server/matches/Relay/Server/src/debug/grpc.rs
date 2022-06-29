use std::sync::{Arc, Mutex};

use cheetah_libraries_microservice::tonic::{Request, Response};
use cheetah_libraries_microservice::trace::trace_and_convert_to_tonic_internal_status_with_full_message;

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
		manager
			.get_rooms()
			.map_err(trace_and_convert_to_tonic_internal_status_with_full_message)
			.map(|rooms| Response::new(admin::GetRoomsResponse { rooms }))
	}
}
