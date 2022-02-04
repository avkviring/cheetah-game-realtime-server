use tonic::Request;

use proto::cerberus::external::{RefreshTokenRequest, RefreshTokenResponse};

use crate::proto;

use super::service::JWTTokensService;
use super::storage::RedisRefreshTokenStorage;

const HOUR_IN_SEC: u64 = 60 * 60;
const SESSION_EXP_IN_SEC: u64 = 10 * HOUR_IN_SEC;
const REFRESH_EXP_IN_SEC: u64 = 30 * 24 * HOUR_IN_SEC;

pub struct TokensGrpcService {
	service: JWTTokensService,
}

impl TokensGrpcService {
	pub fn new(private_key: String, public_key: String, redis_host: String, redis_port: u16, redis_auth: Option<String>) -> Self {
		let storage =
			RedisRefreshTokenStorage::new(redis_host, redis_port, redis_auth, REFRESH_EXP_IN_SEC + HOUR_IN_SEC).unwrap();
		Self {
			service: JWTTokensService::new(private_key, public_key, SESSION_EXP_IN_SEC, REFRESH_EXP_IN_SEC, storage),
		}
	}
}
#[tonic::async_trait]
impl proto::cerberus::external::tokens_server::Tokens for TokensGrpcService {
	async fn refresh(
		&self,
		request: tonic::Request<RefreshTokenRequest>,
	) -> Result<tonic::Response<RefreshTokenResponse>, tonic::Status> {
		let request = request.get_ref();

		match self.service.refresh(request.token.clone()).await {
			Ok(tokens) => Result::Ok(tonic::Response::new(RefreshTokenResponse {
				session: tokens.session,
				refresh: tokens.refresh,
			})),
			Err(e) => {
				log::error!("{:?}", e);
				Result::Err(tonic::Status::unauthenticated(format!("{:?}", e)))
			}
		}
	}
}
