use std::net::SocketAddr;

use cheetah_microservice::proto::auth::user::internal::{
    user_server, CreateRequest, CreateResponse,
};
use cheetah_microservice::tonic::{self, transport::Server, Request, Response, Status};

use crate::storage::Storage;

pub mod storage;

pub async fn run_grpc_server(pool: sqlx::PgPool, binding_addr: SocketAddr) {
    Server::builder()
        .add_service(Service::new(pool).server())
        .serve(binding_addr)
        .await
        .unwrap();
}

pub struct Service {
    storage: Storage,
}

impl Service {
    pub fn new(storage: impl Into<Storage>) -> Self {
        let storage = storage.into();
        Self { storage }
    }

    pub fn server(self) -> user_server::UserServer<Self> {
        user_server::UserServer::new(self)
    }
}

#[tonic::async_trait]
impl user_server::User for Service {
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        let ip = request.get_ref().ip.parse();
        let ip = ip.map_err(|_| Status::invalid_argument("ip address can't parsed"))?;
        let id = self.storage.create(ip).await.into();
        Ok(Response::new(CreateResponse { id }))
    }
}
