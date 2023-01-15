use tonic::transport::Uri;
use tonic::{Request, Status};

use crate::agones::proto::status;
use crate::agones::proto::status::Addr;
use crate::agones::proto::status::State;

pub struct RegistryClient {
	client: status::status_receiver_client::StatusReceiverClient<tonic::transport::Channel>,
}

impl RegistryClient {
	pub async fn new(uri: Uri) -> Result<Self, tonic::transport::Error> {
		let client = status::status_receiver_client::StatusReceiverClient::connect(uri).await?;
		Ok(Self { client })
	}

	pub async fn update_server_status(&self, game: Addr, grpc_internal: Addr, state: State) -> Result<(), Status> {
		let req = Request::new(status::ServerStatus {
			game: Some(game),
			grpc_internal: Some(grpc_internal),
			state: state as i32,
		});

		self.client.clone().set_status(req).await.map(|_| ())
	}
}
