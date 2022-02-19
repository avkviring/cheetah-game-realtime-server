use proto::RefreshTokenRequest;

use crate::proto;
use crate::proto::SessionAndRefreshTokens;
use crate::tokens::TokensService;

pub struct TokensGrpcService {
	service: TokensService,
}

impl TokensGrpcService {
	pub fn new(service: TokensService) -> Self {
		Self { service }
	}
}
#[tonic::async_trait]
impl proto::tokens_server::Tokens for TokensGrpcService {
	async fn refresh(
		&self,
		request: tonic::Request<RefreshTokenRequest>,
	) -> Result<tonic::Response<SessionAndRefreshTokens>, tonic::Status> {
		let request = request.get_ref();

		match self.service.refresh(request.token.clone()).await {
			Ok(tokens) => Result::Ok(tonic::Response::new(SessionAndRefreshTokens {
				session: tokens.session,
				refresh: tokens.refresh,
			})),
			Err(e) => {
				tracing::error!("{:?}", e);
				Result::Err(tonic::Status::unauthenticated(format!("{:?}", e)))
			}
		}
	}
}
