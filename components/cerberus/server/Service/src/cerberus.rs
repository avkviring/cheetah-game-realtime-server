use tonic::Request;

use crate::proto;
use crate::storage::RedisRefreshTokenStorage;
use crate::token::JWTTokensService;

const HOUR_IN_SEC: i64 = 60 * 60;
const SESSION_EXP_IN_SEC: i64 = 5 * HOUR_IN_SEC;
const REFRESH_EXP_IN_SEC: i64 = 30 * 24 * HOUR_IN_SEC;

pub struct Cerberus {
    service: JWTTokensService,
}

impl Cerberus {
    pub fn new(
        private_key: String,
        public_key: String,
        redis_host: String,
        redis_port: u16,
    ) -> Self {
        let storage =
            RedisRefreshTokenStorage::new(redis_host, redis_port, REFRESH_EXP_IN_SEC + HOUR_IN_SEC)
                .unwrap();
        Self {
            service: JWTTokensService::new(
                private_key,
                public_key,
                SESSION_EXP_IN_SEC,
                REFRESH_EXP_IN_SEC,
                storage,
            ),
        }
    }
}

#[tonic::async_trait]
impl proto::internal::cerberus_server::Cerberus for Cerberus {
    async fn create(
        &self,
        request: Request<proto::internal::CreateTokenRequest>,
    ) -> Result<tonic::Response<proto::types::TokensReply>, tonic::Status> {
        let request = request.get_ref();
        match self
            .service
            .create(request.player, request.device_id.clone())
            .await
        {
            Ok(tokens) => Result::Ok(tonic::Response::new(proto::types::TokensReply {
                session: tokens.session,
                refresh: tokens.refresh,
            })),
            Err(e) => {
                log::error!("{:?}", e);
                Result::Err(tonic::Status::failed_precondition(format!("{:?}", e)))
            }
        }
    }
}

#[tonic::async_trait]
impl proto::external::cerberus_server::Cerberus for Cerberus {
    async fn refresh(
        &self,
        request: tonic::Request<proto::external::RefreshTokenRequest>,
    ) -> Result<tonic::Response<proto::types::TokensReply>, tonic::Status> {
        let request = request.get_ref();

        match self.service.refresh(request.token.clone()).await {
            Ok(tokens) => Result::Ok(tonic::Response::new(proto::types::TokensReply {
                session: tokens.session,
                refresh: tokens.refresh,
            })),
            Err(e) => {
                log::error!("{:?}", e);
                Result::Err(tonic::Status::failed_precondition(format!("{:?}", e)))
            }
        }
    }
}
