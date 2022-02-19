use jsonwebtoken_google::Parser;
use sqlx::types::ipnetwork::IpNetwork;
use tonic::{self, Request, Response, Status};

use cheetah_microservice::jwt::JWTTokenParser;

use crate::google::storage::GoogleStorage;
use crate::proto::SessionAndRefreshTokens;
use crate::tokens::TokensService;
use crate::users::{UserId, UserService};
use crate::{get_client_ip, proto};

pub mod storage;

#[derive(serde::Deserialize, serde::Serialize)]
struct GoogleTokenClaim {
	sub: String,
}

pub struct GoogleGrpcService {
	storage: GoogleStorage,
	tokens_service: TokensService,
	users_service: UserService,
	parser: Parser,
	jwt_token_parser: JWTTokenParser,
}

impl GoogleGrpcService {
	pub fn new(
		storage: GoogleStorage,
		tokens_service: TokensService,
		users_service: UserService,
		parser: Parser,
		jwt_token_parser: JWTTokenParser,
	) -> Self {
		Self {
			storage,
			tokens_service,
			users_service,
			parser,
			jwt_token_parser,
		}
	}

	async fn get_or_create_user(&self, ip: IpNetwork, google_id: &str) -> Result<(UserId, bool), Status> {
		if let Some(user) = self.storage.find(google_id).await {
			Ok((user, false))
		} else {
			let user = self.users_service.create(ip).await;
			self.storage.attach(user, google_id, ip).await;
			Ok((user, true))
		}
	}
}

#[tonic::async_trait]
impl proto::google_server::Google for GoogleGrpcService {
	async fn register_or_login(
		&self,
		request: Request<proto::RegisterOrLoginRequest>,
	) -> Result<Response<proto::RegisterOrLoginResponse>, Status> {
		let registry_or_login_request = request.get_ref();
		let token = &registry_or_login_request.google_token;
		let token = self.parser.parse(token).await;
		let GoogleTokenClaim { sub: google_id } = token.map_err(|err| {
			tracing::error!("{:?}", err);
			Status::unauthenticated(format!("{:?}", err))
		})?;

		let ip = get_client_ip(request.metadata());
		let (user, registered_player) = self.get_or_create_user(ip, &google_id).await?;
		let device_id = &registry_or_login_request.device_id;

		let tokens = self.tokens_service.create(user, device_id).await;
		let tokens = tokens.map_err(|err| {
			tracing::error!("{:?}", err);
			Status::internal("error")
		})?;

		Ok(Response::new(proto::RegisterOrLoginResponse {
			registered_player,
			tokens: Some(SessionAndRefreshTokens {
				session: tokens.session,
				refresh: tokens.refresh,
			}),
		}))
	}

	async fn attach(&self, request: Request<proto::AttachRequest>) -> Result<Response<proto::AttachResponse>, Status> {
		let attach_request = request.get_ref();
		let token = &attach_request.google_token;
		let token = self.parser.parse(token).await;
		let GoogleTokenClaim { sub: google_id } = token.map_err(|err| {
			tracing::error!("{:?}", err);
			Status::internal("error")
		})?;

		let user = self
			.jwt_token_parser
			.parse_player_id(request.metadata())
			.map(UserId::from)
			.map_err(|err| {
				tracing::error!("{:?}", err);
				Status::unauthenticated(format!("{:?}", err))
			})?;

		let ip = get_client_ip(request.metadata());
		self.storage.attach(user, &google_id, ip).await;

		Ok(Response::new(proto::AttachResponse {}))
	}
}

#[cfg(test)]
mod test {
	use std::time::Duration;

	use jsonwebtoken_google::test_helper::TokenClaims;
	use jsonwebtoken_google::Parser;
	use testcontainers::clients::Cli;
	use tonic::metadata::MetadataValue;
	use tonic::Request;

	use crate::cookie::CookieGrpcService;
	use crate::google::storage::GoogleStorage;
	use crate::google::GoogleGrpcService;
	use crate::postgresql::test::setup_postgresql_storage;
	use crate::proto::cookie_server::Cookie;
	use crate::proto::google_server::Google;
	use crate::proto::{AttachRequest, RegisterOrLoginRequest, RegistryRequest};
	use crate::tokens::tests::{stub_token_service, PUBLIC_KEY};
	use crate::tokens::TokensService;
	use crate::users::UserService;
	use crate::PgPool;

	#[tokio::test]
	async fn should_register_and_login() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let (_node, token_service) = stub_token_service(1, 100).await;
		let (google_user_token, google_service) = setup_google(pool, token_service);

		let response_1 = google_service
			.register_or_login(Request::new(RegisterOrLoginRequest {
				google_token: google_user_token.clone(),
				device_id: "some-device".to_string(),
			}))
			.await;

		let response_2 = google_service
			.register_or_login(Request::new(RegisterOrLoginRequest {
				google_token: google_user_token.clone(),
				device_id: "some-device".to_string(),
			}))
			.await;
		let result_1 = response_1.unwrap();
		let result_1 = result_1.get_ref();
		let result_2 = response_2.unwrap();
		let result_2 = result_2.get_ref();

		assert!(result_1.registered_player);
		assert!(!result_2.registered_player);

		let jwt = cheetah_microservice::jwt::JWTTokenParser::new(PUBLIC_KEY.to_string());
		let user_1 = jwt.get_user_id(result_1.tokens.as_ref().unwrap().session.clone()).unwrap();
		let user_2 = jwt.get_user_id(result_2.tokens.as_ref().unwrap().session.clone()).unwrap();
		assert_eq!(user_1, user_2);
	}

	fn setup_google(pool: PgPool, token_service: TokensService) -> (String, GoogleGrpcService) {
		let (token, public_key_server) =
			jsonwebtoken_google::test_helper::setup_public_key_server(&TokenClaims::new_with_expire(Duration::from_secs(100)));

		let jwt = cheetah_microservice::jwt::JWTTokenParser::new(PUBLIC_KEY.to_string());
		let service = GoogleGrpcService::new(
			GoogleStorage::new(pool.clone()),
			token_service,
			UserService::new(pool),
			Parser::new_with_custom_cert_url(
				jsonwebtoken_google::test_helper::CLIENT_ID,
				public_key_server.url("/").as_str(),
			),
			jwt,
		);
		(token, service)
	}

	#[tokio::test]
	async fn should_attach() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let (_node, token_service) = stub_token_service(1, 100).await;
		let (google_user_token, google_service) = setup_google(pool.clone(), token_service.clone());
		let service = CookieGrpcService::new(pool.clone(), token_service, UserService::new(pool.clone()));
		let cookie_registry_response = service
			.register(Request::new(RegistryRequest {
				device_id: "some-device".to_string(),
			}))
			.await;

		let cookie_registry_result = cookie_registry_response.unwrap();
		let cookie_registry_result = cookie_registry_result.get_ref();

		let mut request = tonic::Request::new(AttachRequest {
			google_token: google_user_token.clone(),
			device_id: "some-device-id".to_owned(),
		});
		request.metadata_mut().insert(
			"authorization",
			MetadataValue::from_str(&format!("Bearer {}", cookie_registry_result.tokens.as_ref().unwrap().session)).unwrap(),
		);

		google_service.attach(request).await.unwrap();

		let google_login_response = google_service
			.register_or_login(Request::new(RegisterOrLoginRequest {
				google_token: google_user_token.clone(),
				device_id: "some-device-id".to_string(),
			}))
			.await;

		let google_login_result = google_login_response.unwrap();
		let google_login_result = google_login_result.get_ref();

		let jwt = cheetah_microservice::jwt::JWTTokenParser::new(PUBLIC_KEY.to_string());
		let cookie_user_id = jwt
			.get_user_id(cookie_registry_result.tokens.as_ref().unwrap().session.to_string())
			.unwrap();

		let google_user_id = jwt
			.get_user_id(google_login_result.tokens.as_ref().unwrap().session.to_string())
			.unwrap();

		assert_eq!(cookie_user_id, google_user_id);
	}
}
