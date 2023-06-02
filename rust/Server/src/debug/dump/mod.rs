use std::sync::Arc;

use cheetah_protocol::trace::Trace;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::debug::proto::admin;
use crate::server::manager::RoomsServerManager;

pub mod convert;

pub struct DumpGrpcService {
	pub manager: Arc<Mutex<RoomsServerManager>>,
}

impl DumpGrpcService {
	#[must_use]
	pub fn new(manager: Arc<Mutex<RoomsServerManager>>) -> Self {
		Self { manager }
	}
}

#[tonic::async_trait]
impl admin::dump_server::Dump for DumpGrpcService {
	async fn dump(&self, request: Request<admin::DumpRequest>) -> Result<Response<admin::DumpResponse>, Status> {
		let manager = self.manager.lock().await;
		let dump = manager.dump(request.get_ref().room).trace_err("Failed to make a room dump").map_err(Status::internal)?;
		Ok(Response::new(dump))
	}
}
