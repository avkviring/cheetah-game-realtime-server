use tonic::{Request, Response, Status};

use crate::proto::internal;
use crate::registry::allocator::allocate_game_server;

pub mod allocator;
pub mod spec;

pub struct RegistryService {
	address: String,
	port: u32,
}

impl RegistryService {
	pub async fn new() -> Result<RegistryService, Box<dyn std::error::Error>> {
		let status = allocate_game_server().await?;
		let status_port = status.ports.first().unwrap();
		Result::Ok(RegistryService {
			address: status.address,
			port: status_port.port,
		})
	}
}

#[tonic::async_trait]
impl internal::registry_server::Registry for RegistryService {
	async fn find_free_relay(
		&self,
		_request: Request<internal::FindFreeRelayRequest>,
	) -> Result<Response<internal::FindFreeRelayResponse>, Status> {
		Result::Ok(Response::new(internal::FindFreeRelayResponse {
			relay_grpc_host: "".to_string(),
			relay_grpc_port: 0,
			relay_game_host: self.address.clone(),
			relay_game_port: self.port,
		}))
	}
}
