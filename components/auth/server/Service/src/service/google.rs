use jsonwebtoken_google::ParserError;
use serde::Deserialize;
use serde::Serialize;
use tonic::{Request, Response, Status};

use games_cheetah_cerberus_library::grpc::AuthorizationError;

pub use crate::proto::auth::external::google::*;
use crate::proto::cerberus::types::Tokens;
use crate::service::{create_cerberus_token, get_client_ip};
use crate::storage::storage::Storage;
use crate::storage::{google, players};

pub struct GoogleService {
    storage: Storage,
    cerberus_internal_url: String,
    google_token_parser: jsonwebtoken_google::Parser,
    public_jwt_key: String,
}
#[derive(Deserialize, Serialize)]
struct GoogleTokenClaim {
    email: String,
}
impl GoogleService {
    pub fn new(
        storage: Storage,
        cerberus_internal_url: &str,
        google_token_parser: jsonwebtoken_google::Parser,
        public_jwt_key: String,
    ) -> Self {
        Self {
            storage,
            cerberus_internal_url: cerberus_internal_url.to_owned(),
            google_token_parser,
            public_jwt_key,
        }
    }
}

#[tonic::async_trait]
impl google_server::Google for GoogleService {
    async fn registry_or_login(
        &self,
        request: Request<RegistryOrLoginRequest>,
    ) -> Result<Response<Tokens>, tonic::Status> {
        let registry_or_login_request = request.get_ref();
        let google_token = registry_or_login_request.google_token.as_str();
        match self
            .google_token_parser
            .parse::<GoogleTokenClaim>(google_token)
            .await
        {
            Ok(token) => {
                let email = token.email.as_str();
                let player = self.get_or_create_player(&request, email).await;
                let token = create_cerberus_token(
                    self.cerberus_internal_url.to_owned(),
                    player,
                    registry_or_login_request.device_id.to_owned(),
                )
                .await;
                match token {
                    Ok(token) => Result::Ok(Response::new(token.into_inner())),
                    Err(e) => {
                        log::error!("{:?}", e);
                        Result::Err(tonic::Status::internal("error"))
                    }
                }
            }
            Err(e) => {
                log::error!("{:?}", e);
                Result::Err(tonic::Status::unauthenticated(format!("{:?}", e)))
            }
        }
    }

    async fn attach(
        &self,
        request: Request<AttachRequest>,
    ) -> Result<Response<AttachResponse>, tonic::Status> {
        let attach_request = request.get_ref();
        let google_token = attach_request.google_token.as_str();
        match self
            .google_token_parser
            .parse::<GoogleTokenClaim>(google_token)
            .await
        {
            Ok(token) => {
                match games_cheetah_cerberus_library::grpc::get_player_id(
                    request.metadata(),
                    self.public_jwt_key.to_owned(),
                ) {
                    Ok(player) => {
                        google::attach(
                            &self.storage,
                            player,
                            token.email.as_str(),
                            &get_client_ip(request.metadata()),
                        )
                        .await;
                        Result::Ok(tonic::Response::new(AttachResponse {}))
                    }
                    Err(_) => Result::Err(tonic::Status::unauthenticated("error")),
                }
            }
            Err(e) => {
                log::error!("{:?}", e);
                Result::Err(tonic::Status::internal("error"))
            }
        }
    }
}

impl GoogleService {
    async fn get_or_create_player(
        &self,
        request: &Request<RegistryOrLoginRequest>,
        email: &str,
    ) -> u64 {
        let player = match google::find(&self.storage, email).await {
            None => {
                let ip = get_client_ip(request.metadata());
                players::create_player(&self.storage, &ip).await
            }
            Some(player) => player,
        };
        player
    }
}
