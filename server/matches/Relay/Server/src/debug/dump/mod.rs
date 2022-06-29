use std::sync::{Arc, Mutex};

use cheetah_libraries_microservice::tonic::{Request, Response};
use cheetah_libraries_microservice::trace::trace_and_convert_to_tonic_internal_status_with_full_message;

use crate::debug::proto::admin;
use crate::server::manager::ServerManager;

pub mod convert;

pub struct DumpGrpcService {
	pub manager: Arc<Mutex<ServerManager>>,
}

impl DumpGrpcService {
	pub fn new(manager: Arc<Mutex<ServerManager>>) -> Self {
		Self { manager }
	}
}

#[tonic::async_trait]
impl admin::dump_server::Dump for DumpGrpcService {
	async fn dump(
		&self,
		request: Request<admin::DumpRequest>,
	) -> Result<Response<admin::DumpResponse>, tonic::Status> {
		let manager = self.manager.lock().unwrap();
		let dump = manager
			.dump(request.get_ref().room)
			.map_err(trace_and_convert_to_tonic_internal_status_with_full_message)?;
		Ok(Response::new(dump))
	}
}
