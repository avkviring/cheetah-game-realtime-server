use std::time::Duration;

use tonic::{Request, Response, Status};

pub struct RegistryService {}

impl RegistryService {
	pub fn allocate_game_server(&self) {}
}

#[tonic::async_trait]
impl crate::proto::internal::registry_server::Registry for RegistryService {
	async fn find_free_relay(
		&self,
		_request: Request<crate::proto::internal::FindFreeRelayRequest>,
	) -> Result<Response<crate::proto::internal::FindFreeRelayResponse>, Status> {
		todo!()
	}
}
