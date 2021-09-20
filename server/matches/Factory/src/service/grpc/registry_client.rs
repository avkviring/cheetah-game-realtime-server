use std::convert::TryInto;

use cheetah_microservice::tonic::transport::Endpoint;

use crate::proto::matches::registry::internal as registry;

pub struct RegistryClient {
	endpoint: Endpoint,
}

impl RegistryClient {
	pub fn new<E: TryInto<Endpoint>>(endpoint: E) -> Result<Self, E::Error> {
		endpoint.try_into().map(|endpoint| Self { endpoint })
	}

	pub async fn find_free_relay(&self) -> registry::FindFreeRelayResponse {
		registry::registry_client::RegistryClient::connect(self.endpoint.clone())
			.await
			.unwrap()
			.find_free_relay(registry::FindFreeRelayRequest {})
			.await
			.unwrap()
			.into_inner()
	}
}
