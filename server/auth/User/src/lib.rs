use std::net::SocketAddr;

use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::proto::auth::user::internal;
use crate::storage::Storage;

pub mod proto;
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

	pub fn server(self) -> internal::user_server::UserServer<Self> {
		internal::user_server::UserServer::new(self)
	}
}

#[tonic::async_trait]
impl internal::user_server::User for Service {
	async fn create(&self, request: Request<internal::CreateRequest>) -> Result<Response<internal::CreateResponse>, Status> {
		let ip = request.get_ref().ip.parse();
		let ip = ip.map_err(|_| Status::invalid_argument("ip address can't parsed"))?;
		let id = self.storage.create(ip).await.into();
		Ok(Response::new(internal::CreateResponse { id }))
	}
}
