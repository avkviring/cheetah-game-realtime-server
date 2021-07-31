use sqlx::types::ipnetwork::IpNetwork;
use tonic::codegen::http::Uri;

use cheetah_microservice::{
    proto::auth::cookie::external::{
        cookie_server,
        {login_response::Status as LoginStatus, LoginRequest, LoginResponse},
        {RegistryRequest, RegistryResponse},
    },
    tonic::{self, transport::Server, Request, Response},
};

use crate::api::{cerberus, user};
use crate::storage::{FindResult, Storage};
use std::net::SocketAddr;

pub mod api;
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

    Server::builder()
        .add_service(Service::new(pool.clone(), cerberus.clone(), user_client.clone()).server())
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
    pub fn new(
        storage: impl Into<Storage>,
        cerberus: cerberus::Client,
        users: user::Client,
    ) -> Self {
        Self {
            storage: storage.into(),
            cerberus,
            users,
        }
    }

    pub fn server(self) -> cookie_server::CookieServer<Self> {
        cookie_server::CookieServer::new(self)
    }
}

#[tonic::async_trait]
impl cookie_server::Cookie for Service {
    async fn register(
        &self,
        request: Request<RegistryRequest>,
    ) -> Result<Response<RegistryResponse>, tonic::Status> {
        let ip = get_client_ip(request.metadata());
        let user = self.users.create(ip).await?;
        let cookie = self.storage.attach(user).await;
        self.cerberus
            .create_token(&request.get_ref().device_id, user)
            .await
            .map(Some)
            .map(|tokens| RegistryResponse { tokens, cookie })
            .map(Response::new)
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, tonic::Status> {
        let request = request.get_ref();
        match self.storage.find(&request.cookie).await {
            FindResult::NotFound => Ok((None, LoginStatus::NotFound as i32)),
            FindResult::Linked => Ok((None, LoginStatus::Linked as i32)),
            FindResult::Player(user) => self
                .cerberus
                .create_token(&request.device_id, user)
                .await
                .map(|tokens| (Some(tokens), LoginStatus::Ok as i32)),
        }
        .map(|(tokens, status)| LoginResponse { tokens, status })
        .map(Response::new)
    }
}
