use cheetah_libraries_microservice::trace::trace_error_and_convert_to_unauthenticated_tonic_status;
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

		let tokens = self
			.service
			.refresh(request.token.clone())
			.await
			.map_err(trace_error_and_convert_to_unauthenticated_tonic_status)?;

		Ok(tonic::Response::new(SessionAndRefreshTokens {
			session: tokens.session,
			refresh: tokens.refresh,
		}))
	}
}
