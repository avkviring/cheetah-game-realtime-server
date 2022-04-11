use crate::agones::proto::registry;
use crate::agones::proto::registry::RelayAddrs;
use crate::agones::proto::registry::RelayState;
use tonic::transport::Uri;
use tonic::{Request, Status};

pub struct RegistryClient {
	client: registry::registry_client::RegistryClient<tonic::transport::Channel>,
}

impl RegistryClient {
	pub async fn new(uri: Uri) -> Result<Self, tonic::transport::Error> {
		let client = registry::registry_client::RegistryClient::connect(uri).await?;

		Ok(Self { client })
	}

	pub async fn update_relay_status(&self, addrs: RelayAddrs, state: RelayState) -> Result<(), Status> {
		let req = Request::new(registry::RelayStatusUpdate {
			addrs: Some(addrs),
			state: state as i32,
		});

		self.client.clone().update_relay_status(req).await.map(|_| ())
	}
}
