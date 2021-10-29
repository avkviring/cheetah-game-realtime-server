use std::sync::{Arc, Mutex};

use cheetah_microservice::tonic::{Request, Response};

use crate::debug::proto::admin;
use crate::server::manager::RelayManager;

pub mod convert;

pub struct DumpGrpcService {
	pub manager: Arc<Mutex<RelayManager>>,
}

impl DumpGrpcService {
	pub fn new(manager: Arc<Mutex<RelayManager>>) -> Self {
		Self { manager }
	}
}

#[tonic::async_trait]
impl admin::dump_server::Dump for DumpGrpcService {
	async fn dump(&self, request: Request<admin::DumpRequest>) -> Result<Response<admin::DumpResponse>, tonic::Status> {
		let manager = self.manager.lock().unwrap();
		match manager.dump(request.get_ref().room) {
			Ok(dump) => Result::Ok(Response::new(dump)),
			Err(e) => Result::Err(tonic::Status::internal(format!("{:?}", e))),
		}
	}
}
