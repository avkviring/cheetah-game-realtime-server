use sqlx::types::ipnetwork::IpNetwork;

use cheetah_microservice::proto::auth::user::internal::{
    user_client, CreateRequest, CreateResponse,
};
use cheetah_microservice::tonic::{Request, Response, Status};
use cheetah_microservice::tonic;

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::FromRow, sqlx::Type)]
#[sqlx(transparent)]
pub struct Id(i64);

impl From<Id> for u64 {
    fn from(id: Id) -> u64 {
        id.0 as u64
    }
}

impl From<u64> for Id {
    fn from(id: u64) -> Self {
        Self(id as i64)
    }
}

#[derive(Clone)]
pub struct Client {
    addr: tonic::transport::Endpoint,
}

impl Client {
    pub fn new(addr: impl Into<tonic::transport::Endpoint>) -> Self {
        Self { addr: addr.into() }
    }

    pub async fn create(&self, ip: IpNetwork) -> Result<Id, Status> {
        let ip = ip.to_string();
        user_client::UserClient::connect(self.addr.clone())
            .await
            .unwrap()
            .create(Request::new(CreateRequest { ip }))
            .await
            .map(Response::into_inner)
            .map(|CreateResponse { id, .. }| Id(id))
    }
}
