pub mod api;
pub mod proto;
pub mod storage;

use crate::api::{cerberus, user};
use crate::proto::auth::google::external::{
    google_server, {AttachRequest, AttachResponse},
    {RegisterOrLoginRequest, RegisterOrLoginResponse},
};
use crate::storage::Storage;
use cheetah_microservice::jwt::JWTTokenParser;
use jsonwebtoken_google::Parser as GoogleTokenParser;
use sqlx::types::ipnetwork::IpNetwork;
use tonic::{transport::Server, Request, Response};

pub fn get_client_ip(metadata: &tonic::metadata::MetadataMap) -> IpNetwork {
    metadata
        .get("X-Forwarded-For")
        .and_then(|x_forwarder_for| x_forwarder_for.to_str().ok())
        .and_then(|peer_ip| peer_ip.parse().ok())
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap())
}

pub async fn run_grpc_server(
    pool: sqlx::PgPool,
    service_port: u16,
    cerberus_url: impl Into<String>,
    user_url: impl Into<String>,
    public_jwt_key: impl Into<String>,
    parser: GoogleTokenParser,
) {
    let cerberus = cerberus::Client::new(cerberus_url);
    let users = user::Client::new(user_url);
    let jwt = JWTTokenParser::new(public_jwt_key.into());

    Server::builder()
        .add_service(Service::new(pool, cerberus, users, parser, jwt).server())
        .serve(format!("0.0.0.0:{}", service_port).parse().unwrap())
        .await
        .unwrap();
}

#[derive(serde::Deserialize, serde::Serialize)]
struct GoogleTokenClaim {
    sub: String,
}

pub struct Service {
    storage: Storage,
    cerberus: cerberus::Client,
    users: user::Client,
    parser: GoogleTokenParser,
    jwt: JWTTokenParser,
}

impl Service {
    pub fn new(
        storage: impl Into<Storage>,
        cerberus: cerberus::Client,
        users: user::Client,
        parser: GoogleTokenParser,
        jwt: JWTTokenParser,
    ) -> Self {
        Self {
            storage: storage.into(),
            cerberus,
            users,
            parser,
            jwt,
        }
    }

    pub fn server(self) -> google_server::GoogleServer<Self> {
        google_server::GoogleServer::new(self)
    }

    async fn get_or_create_user(
        &self,
        ip: IpNetwork,
        google_id: &str,
    ) -> Result<(user::Id, bool), tonic::Status> {
        if let Some(user) = self.storage.find(google_id).await {
            Ok((user, false))
        } else {
            let user = self.users.create(ip).await?;
            self.storage.attach(user, google_id, ip).await;
            Ok((user, true))
        }
    }
}

#[tonic::async_trait]
impl google_server::Google for Service {
    async fn register_or_login(
        &self,
        request: Request<RegisterOrLoginRequest>,
    ) -> Result<Response<RegisterOrLoginResponse>, tonic::Status> {
        let registry_or_login_request = request.get_ref();
        let token = &registry_or_login_request.google_token;
        let token = self.parser.parse(token).await;
        let GoogleTokenClaim { sub: google_id } = token.map_err(|err| {
            log::error!("{:?}", err);
            tonic::Status::unauthenticated(format!("{:?}", err))
        })?;

        let ip = get_client_ip(request.metadata());
        let (user, registered_player) = self.get_or_create_user(ip, &google_id).await?;
        let device_id = &registry_or_login_request.device_id;

        let tokens = self.cerberus.create_token(device_id, user).await;
        let tokens = tokens.map_err(|err| {
            log::error!("{:?}", err);
            tonic::Status::internal("error")
        })?;

        Ok(Response::new(RegisterOrLoginResponse {
            registered_player,
            tokens: Some(tokens),
        }))
    }

    async fn attach(
        &self,
        request: Request<AttachRequest>,
    ) -> Result<Response<AttachResponse>, tonic::Status> {
        let attach_request = request.get_ref();
        let token = &attach_request.google_token;
        let token = self.parser.parse(token).await;
        let GoogleTokenClaim { sub: google_id } = token.map_err(|err| {
            log::error!("{:?}", err);
            tonic::Status::internal("error")
        })?;

        let user = self
            .jwt
            .parse_player_id(request.metadata())
            .map(user::Id::from)
            .map_err(|err| {
                log::error!("{:?}", err);
                tonic::Status::unauthenticated(format!("{:?}", err))
            })?;

        let ip = get_client_ip(request.metadata());
        self.storage.attach(user, &google_id, ip).await;

        Ok(tonic::Response::new(AttachResponse {}))
    }
}
