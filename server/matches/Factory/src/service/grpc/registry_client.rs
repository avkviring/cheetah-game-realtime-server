use crate::proto::matches::registry::internal as registry;
use crate::proto::matches::registry::internal::RelayAddrs;
use thiserror::Error;
use tonic::transport::Uri;

#[derive(Error, Debug)]
pub enum RegistryError {
	#[error("RelayAddrs field is empty")]
	RelayAddrsFieldIsEmpty,
	#[error(transparent)]
	CouldNotConnect(#[from] tonic::transport::Error),
	#[error(transparent)]
	RpcFailed(#[from] tonic::Status),
}

pub struct RegistryClient {
	client: registry::registry_client::RegistryClient<tonic::transport::Channel>,
}

impl RegistryClient {
	pub async fn new(uri: Uri) -> Result<Self, RegistryError> {
		let client = registry::registry_client::RegistryClient::connect(uri).await?;

		Ok(Self { client })
	}

	pub async fn find_free_relay(&self) -> Result<RelayAddrs, RegistryError> {
		self.client
			.clone()
			.find_free_relay(registry::FindFreeRelayRequest {})
			.await
			.map_err(RegistryError::from)?
			.into_inner()
			.addrs
			.ok_or(RegistryError::RelayAddrsFieldIsEmpty)
	}
}
