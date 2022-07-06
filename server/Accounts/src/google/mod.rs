use jwt_tonic_user_uuid::JWTUserTokenParser;
use tonic::{self, Request, Response, Status};

use cheetah_libraries_microservice::trace::ResultErrorTracer;
use google_jwt::Parser;

use crate::google::storage::GoogleStorage;
use crate::proto;
use crate::proto::SessionAndRefreshTokens;
use crate::tokens::TokensService;
use crate::users::service::UserService;
use crate::users::user::User;

pub mod google_jwt;
pub mod storage;
#[cfg(test)]
pub mod test_helper;

#[derive(serde::Deserialize, serde::Serialize)]
struct GoogleTokenClaim {
	sub: String,
}

pub struct GoogleGrpcService {
	storage: GoogleStorage,
	tokens_service: TokensService,
	users_service: UserService,
	parser: Parser,
	jwt_token_parser: JWTUserTokenParser,
}

impl GoogleGrpcService {
	pub fn new(
		storage: GoogleStorage,
		tokens_service: TokensService,
		users_service: UserService,
		parser: Parser,
		jwt_token_parser: JWTUserTokenParser,
	) -> Self {
		Self {
			storage,
			tokens_service,
			users_service,
			parser,
			jwt_token_parser,
		}
	}

	async fn get_or_create_user(&self, google_id: &str) -> anyhow::Result<(User, bool)> {
		match self.storage.find(google_id).await? {
			None => {
				let user = self.users_service.create().await?;
				self.storage.attach(user, google_id).await?;
				Ok((user, true))
			}
			Some(user) => Ok((user, false)),
		}
	}

	async fn parse_google_id(&self, token: &String) -> Result<String, Status> {
		let GoogleTokenClaim { sub: google_id } = self
			.parser
			.parse(token)
			.await
			.trace_and_map_msg(format!("Parse google id {}", token), |_| {
				Status::internal("")
			})?;
		Ok(google_id)
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
		let google_id = self.parse_google_id(token).await?;

		let (user, registered_user) = self
			.get_or_create_user(&google_id)
			.await
			.trace_and_map_msg(format!("Google get or create user {}", token), |_| {
				Status::internal("")
			})?;

		let device_id = &registry_or_login_request.device_id;

		let tokens = self
			.tokens_service
			.create(user, device_id)
			.await
			.trace_and_map_msg("Create token for user", |_| Status::internal(""))?;

		Ok(Response::new(proto::RegisterOrLoginResponse {
			registered_player: registered_user,
			tokens: Some(SessionAndRefreshTokens {
				session: tokens.session,
				refresh: tokens.refresh,
			}),
		}))
	}

	async fn attach(
		&self,
		request: Request<proto::AttachRequest>,
	) -> Result<Response<proto::AttachResponse>, Status> {
		let attach_request = request.get_ref();
		let token = &attach_request.google_token;
		let google_id = self.parse_google_id(token).await?;

		let user_uuid = self
			.jwt_token_parser
			.parse_user_uuid(request.metadata())
			.trace_and_map_msg(format!("Parse jwt token {:?}", request.metadata()), |_| {
				Status::internal("")
			})?;

		let user = User::try_from(user_uuid)
			.trace_and_map_msg(format!("Convert uuid to user {:?}", user_uuid), |_| {
				Status::internal("")
			})?;

		self.storage.attach(user, &google_id).await.unwrap();

		Ok(Response::new(proto::AttachResponse {}))
	}
}

#[cfg(test)]
mod test {
	use std::time::Duration;

	use tonic::metadata::MetadataValue;
	use tonic::Request;
	use ydb::TableClient;

	use crate::cookie::service::CookieService;
	use crate::google::google_jwt::Parser;
	use crate::google::storage::GoogleStorage;
	use crate::google::test_helper::TokenClaims;
	use crate::google::{test_helper, GoogleGrpcService};
	use crate::proto::cookie_server::Cookie;
	use crate::proto::google_server::Google;
	use crate::proto::{AttachRequest, RegisterOrLoginRequest, RegistryRequest};
	use crate::tokens::tests::{stub_token_service, PUBLIC_KEY};
	use crate::tokens::TokensService;
	use crate::users::service::UserService;
	use crate::ydb::test::setup_ydb;

	#[tokio::test]
	async fn should_register_and_login() {
		let (ydb_client, _instance) = setup_ydb().await;
		let (_node, token_service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(100)).await;
		let (google_user_token, google_service) =
			setup_google(ydb_client.table_client(), token_service);

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

		let jwt = jwt_tonic_user_uuid::JWTUserTokenParser::new(PUBLIC_KEY.to_string());
		let user_1 = jwt
			.get_user_uuid(result_1.tokens.as_ref().unwrap().session.clone())
			.unwrap();
		let user_2 = jwt
			.get_user_uuid(result_2.tokens.as_ref().unwrap().session.clone())
			.unwrap();
		assert_eq!(user_1, user_2);
	}

	fn setup_google(
		ydb_table_client: TableClient,
		token_service: TokensService,
	) -> (String, GoogleGrpcService) {
		let (token, public_key_server) = test_helper::setup_public_key_server(
			&TokenClaims::new_with_expire(Duration::from_secs(100)),
		);

		let jwt = jwt_tonic_user_uuid::JWTUserTokenParser::new(PUBLIC_KEY.to_string());
		let service = GoogleGrpcService::new(
			GoogleStorage::new(ydb_table_client.clone()),
			token_service,
			UserService::new(ydb_table_client),
			Parser::new_with_custom_cert_url(
				test_helper::CLIENT_ID,
				public_key_server.url("/").as_str(),
			),
			jwt,
		);
		(token, service)
	}

	#[tokio::test]
	async fn should_attach() {
		let (ydb_client, _instance) = setup_ydb().await;
		let (_node, token_service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(100)).await;
		let (google_user_token, google_service) =
			setup_google(ydb_client.table_client(), token_service.clone());
		let service = CookieService::new(
			ydb_client.table_client(),
			token_service,
			UserService::new(ydb_client.table_client()),
		);
		let cookie_registry_response = service
			.register(Request::new(RegistryRequest {
				device_id: "some-device".to_string(),
			}))
			.await;

		let cookie_registry_result = cookie_registry_response.unwrap();
		let cookie_registry_result = cookie_registry_result.get_ref();

		let mut request = Request::new(AttachRequest {
			google_token: google_user_token.clone(),
			device_id: "some-device-id".to_owned(),
		});
		request.metadata_mut().insert(
			"authorization",
			MetadataValue::from_str(&format!(
				"Bearer {}",
				cookie_registry_result.tokens.as_ref().unwrap().session
			))
			.unwrap(),
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

		let jwt = jwt_tonic_user_uuid::JWTUserTokenParser::new(PUBLIC_KEY.to_string());
		let cookie_user_id = jwt
			.get_user_uuid(
				cookie_registry_result
					.tokens
					.as_ref()
					.unwrap()
					.session
					.to_string(),
			)
			.unwrap();

		let google_user_id = jwt
			.get_user_uuid(
				google_login_result
					.tokens
					.as_ref()
					.unwrap()
					.session
					.to_string(),
			)
			.unwrap();

		assert_eq!(cookie_user_id, google_user_id);
	}
}
