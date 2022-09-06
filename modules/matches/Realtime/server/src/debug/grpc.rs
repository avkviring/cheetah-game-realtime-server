use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::Status;

use cheetah_libraries_microservice::tonic::{Request, Response};
use cheetah_libraries_microservice::trace::Trace;

use crate::debug::proto::admin;
use crate::server::manager::ServerManager;

pub struct RealtimeAdminGRPCService {
	pub manager: Arc<Mutex<ServerManager>>,
}
impl RealtimeAdminGRPCService {
	pub fn new(manager: Arc<Mutex<ServerManager>>) -> Self {
		Self { manager }
	}
}

#[tonic::async_trait]
impl admin::realtime_server::Realtime for RealtimeAdminGRPCService {
	async fn get_rooms(&self, _request: Request<admin::GetRoomsRequest>) -> Result<Response<admin::GetRoomsResponse>, Status> {
		let manager = self.manager.lock().await;
		manager
			.get_rooms()
			.trace_err("Get rooms")
			.map_err(Status::internal)
			.map(|rooms| Response::new(admin::GetRoomsResponse { rooms }))
	}
}
