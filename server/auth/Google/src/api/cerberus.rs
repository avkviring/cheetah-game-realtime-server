use crate::api::user;
use crate::proto::auth::cerberus::internal::{cerberus_client, CreateTokenRequest};
pub use crate::proto::auth::cerberus::types::Tokens;
use tonic::{Request, Response};

#[derive(Clone)]
pub struct Client {
    addr: String,
}

impl Client {
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }

    pub async fn create_token(
        &self,
        device_id: impl Into<String>,
        player: user::Id,
    ) -> Result<Tokens, tonic::Status> {
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
