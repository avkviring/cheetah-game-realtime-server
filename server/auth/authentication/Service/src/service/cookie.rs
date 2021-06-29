use tonic::{Request, Response};

use auth::external::cookie::*;

use crate::proto::auth;
use crate::proto::cerberus::types::Tokens;
use crate::service::{create_cerberus_token, get_client_ip};
use crate::storage::cookie::FindResult;
use crate::storage::pg::PgStorage;
use crate::storage::{cookie, players};

pub struct CookieService {
    storage: PgStorage,
    cerberus_internal_url: String,
}

impl CookieService {
    pub fn new(storage: PgStorage, cerberus_internal_url: &str) -> Self {
        Self {
            storage,
            cerberus_internal_url: cerberus_internal_url.to_owned(),
        }
    }

    async fn create_token(
        &self,
        device_id: String,
        player: u64,
    ) -> Result<Response<Tokens>, tonic::Status> {
        let cerberus_internal_url = self.cerberus_internal_url.to_owned();
        create_cerberus_token(cerberus_internal_url, player, device_id).await
    }
}

#[tonic::async_trait]
impl cookie_server::Cookie for CookieService {
    async fn register(
        &self,
        request: Request<RegistryRequest>,
    ) -> Result<tonic::Response<RegistryResponse>, tonic::Status> {
        let ip = get_client_ip(&request.metadata());
        let player = players::create_player(&self.storage, &ip).await;
        let cookie = cookie::attach(&self.storage, player).await;
        self.create_token(request.get_ref().device_id.to_owned(), player)
            .await
            .map(|r| {
                Response::new(RegistryResponse {
                    tokens: Some(r.into_inner()),
                    cookie,
                })
            })
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, tonic::Status> {
        let request = request.get_ref();
        let result = cookie::find(&self.storage, request.cookie.as_str()).await;
        match result {
            FindResult::NotFound => Result::Ok(Response::new(LoginResponse {
                tokens: None,
                status: login_response::Status::NotFound as i32,
            })),
            FindResult::Linked => Result::Ok(Response::new(LoginResponse {
                tokens: None,
                status: login_response::Status::Linked as i32,
            })),
            FindResult::Player(player) => self
                .create_token(request.device_id.to_owned(), player)
                .await
                .map(|r| {
                    Response::new(LoginResponse {
                        tokens: Some(r.into_inner()),
                        status: 0,
                    })
                }),
        }
    }
}
