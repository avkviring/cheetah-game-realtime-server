use tonic::{Request, Response, Status};

use crate::proto::internal;
use crate::registry::allocator::allocate_game_server;
use crate::registry::pod::get_pod_ip;

pub mod allocator;
pub mod pod;
pub mod spec;

#[derive(Debug)]
pub struct RegistryService {
	grpc_address: String,
	grpc_port: u16,
	game_address: String,
	game_port: u32,
}

impl RegistryService {
	pub async fn new() -> Result<RegistryService, Box<dyn std::error::Error>> {
		let status = allocate_game_server().await?;
		let status_port = status.ports.first().unwrap();
		let registry_service = RegistryService {
			grpc_address: get_pod_ip(&status.game_server_name).await?,
			grpc_port: cheetah_microservice::get_internal_service_port(),
			game_address: status.address,
			game_port: status_port.port,
		};
		log::info!("RegistryService  {:?}", registry_service);
		Result::Ok(registry_service)
	}
}

#[tonic::async_trait]
impl internal::registry_server::Registry for RegistryService {
	async fn find_free_relay(
		&self,
		_request: Request<internal::FindFreeRelayRequest>,
	) -> Result<Response<internal::FindFreeRelayResponse>, Status> {
		Result::Ok(Response::new(internal::FindFreeRelayResponse {
			relay_grpc_host: self.grpc_address.clone(),
			relay_grpc_port: self.grpc_port as u32,
			relay_game_host: self.game_address.clone(),
			relay_game_port: self.game_port,
		}))
	}
}
