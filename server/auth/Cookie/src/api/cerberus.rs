use crate::api::user;
use cheetah_microservice::proto::auth::cerberus::internal::{cerberus_client, CreateTokenRequest};
pub use cheetah_microservice::proto::auth::cerberus::types::Tokens;
use cheetah_microservice::tonic;
use cheetah_microservice::tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct Client {
	addr: tonic::transport::Endpoint,
}

impl Client {
	pub fn new(addr: impl Into<tonic::transport::Endpoint>) -> Self {
		Self { addr: addr.into() }
	}

	pub async fn create_token(&self, device_id: impl Into<String>, player: user::Id) -> Result<Tokens, Status> {
		let player = player.into();
		let device_id = device_id.into();
		cerberus_client::CerberusClient::connect(self.addr.clone())
			.await
			.unwrap()
			.create(Request::new(CreateTokenRequest { player, device_id }))
			.await
			.map(Response::into_inner)
	}
}
