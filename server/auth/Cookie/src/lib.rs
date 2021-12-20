use std::net::SocketAddr;

use sqlx::types::ipnetwork::IpNetwork;
use tonic::transport::Server;
use tonic::{Request, Response};

use crate::api::{cerberus, user};
use crate::proto::auth::cookie::external;
use crate::storage::{FindResult, Storage};

pub mod api;
pub mod proto;
pub mod storage;

pub fn get_client_ip(metadata: &tonic::metadata::MetadataMap) -> IpNetwork {
	metadata
		.get("X-Forwarded-For")
		.and_then(|x_forwarder_for| x_forwarder_for.to_str().ok())
		.and_then(|peer_ip| peer_ip.parse().ok())
		.unwrap_or_else(|| "127.0.0.1".parse().unwrap())
}

pub async fn run_grpc_server(
	pool: sqlx::PgPool,
	cerberus_internal_service: impl Into<tonic::transport::Endpoint>,
	user_internal_service: impl Into<tonic::transport::Endpoint>,
	binding_address: SocketAddr,
) {
	let cerberus = cerberus::Client::new(cerberus_internal_service);
	let user_client = user::Client::new(user_internal_service);

	let service = Service::new(pool.clone(), cerberus.clone(), user_client.clone()).grpc_service();
	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(service))
		.serve(binding_address)
		.await
		.unwrap();
}

pub struct Service {
	storage: Storage,
	cerberus: cerberus::Client,
	users: user::Client,
}

impl Service {
	pub fn new(storage: impl Into<Storage>, cerberus: cerberus::Client, users: user::Client) -> Self {
		Self {
			storage: storage.into(),
			cerberus,
			users,
		}
	}

	pub fn grpc_service(self) -> external::cookie_server::CookieServer<Self> {
		external::cookie_server::CookieServer::new(self)
	}
}

#[tonic::async_trait]
impl external::cookie_server::Cookie for Service {
	async fn register(
		&self,
		request: Request<external::RegistryRequest>,
	) -> Result<Response<external::RegistryResponse>, tonic::Status> {
		let ip = get_client_ip(request.metadata());
		let user = self.users.create(ip).await?;
		let cookie = self.storage.attach(user).await;
		self.cerberus
			.create_token(&request.get_ref().device_id, user)
			.await
			.map(Some)
			.map(|tokens| external::RegistryResponse { tokens, cookie })
			.map(Response::new)
	}

	async fn login(&self, request: Request<external::LoginRequest>) -> Result<Response<external::LoginResponse>, tonic::Status> {
		let request = request.get_ref();
		match self.storage.find(&request.cookie).await {
			FindResult::NotFound => Ok((None, external::login_response::Status::NotFound as i32)),
			FindResult::Linked => Ok((None, external::login_response::Status::Linked as i32)),
			FindResult::Player(user) => self
				.cerberus
				.create_token(&request.device_id, user)
				.await
				.map(|tokens| (Some(tokens), external::login_response::Status::Ok as i32)),
		}
		.map(|(tokens, status)| external::LoginResponse { tokens, status })
		.map(Response::new)
	}
}
