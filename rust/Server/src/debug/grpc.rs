use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::Status;

use cheetah_microservice::tonic::{Request, Response};
use cheetah_microservice::trace::Trace;

use crate::debug::proto::admin;
use crate::server::manager::RoomsServerManager;

pub struct RealtimeAdminGRPCService {
	pub manager: Arc<Mutex<RoomsServerManager>>,
}
impl RealtimeAdminGRPCService {
	#[must_use]
	pub fn new(manager: Arc<Mutex<RoomsServerManager>>) -> Self {
		Self { manager }
	}
}

#[tonic::async_trait]
impl admin::admin_server::Admin for RealtimeAdminGRPCService {
	async fn get_rooms(&self, _request: Request<admin::GetRoomsRequest>) -> Result<Response<admin::GetRoomsResponse>, Status> {
		let manager = self.manager.lock().await;
		manager
			.get_rooms()
			.trace_err("Get rooms")
			.map_err(Status::internal)
			.map(|rooms| Response::new(admin::GetRoomsResponse { rooms }))
	}
}
