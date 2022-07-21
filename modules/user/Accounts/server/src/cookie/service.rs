use lazy_static::lazy_static;
use prometheus::{register_int_counter, IntCounter};
use tonic::{Request, Response, Status};
use uuid::Uuid;
use ydb::TableClient;

use cheetah_libraries_microservice::trace::Trace;

use crate::cookie::storage::CookieStorage;
use crate::cookie::Cookie;
use crate::proto;
use crate::proto::{LoginRequest, SessionAndRefreshTokens};
use crate::tokens::TokensService;
use crate::users::service::UserService;
use crate::users::user::User;

pub struct CookieService {
	storage: CookieStorage,
	token_service: TokensService,
	user_service: UserService,
}

impl CookieService {
	pub fn new(
		ydb_table_client: TableClient,
		token_service: TokensService,
		user_service: UserService,
	) -> Self {
		Self {
			storage: CookieStorage::new(ydb_table_client),
			token_service,
			user_service,
		}
	}

	async fn do_register(
		&self,
		device_id: &str,
	) -> anyhow::Result<(SessionAndRefreshTokens, Cookie)> {
		let user = self.user_service.create().await?;
		let cookie = self.storage.attach(user).await?;
		let tokens = self.create_jwt_tokens(user, device_id).await?;
		Ok((tokens, cookie))
	}

	async fn do_login(
		&self,
		request: &LoginRequest,
		cookie: Cookie,
	) -> anyhow::Result<Option<SessionAndRefreshTokens>> {
		Ok(match self.storage.find(&cookie).await? {
			None => None,
			Some(user) => Some(
				self.create_jwt_tokens(user, request.device_id.as_str())
					.await?,
			),
		})
	}

	async fn create_jwt_tokens(
		&self,
		user: User,
		device_id: &str,
	) -> anyhow::Result<SessionAndRefreshTokens> {
		let result = self
			.token_service
			.create(user, device_id)
			.await
			.map(|tokens| SessionAndRefreshTokens {
				session: tokens.session,
				refresh: tokens.refresh,
			})?;
		Ok(result)
	}
}

#[tonic::async_trait]
impl proto::cookie_server::Cookie for CookieService {
	async fn register(
		&self,
		request: Request<proto::RegistryRequest>,
	) -> Result<Response<proto::RegistryResponse>, Status> {
		COOKIE_REGISTER_COUNTER.inc();
		let device_id = &request.get_ref().device_id;
		self.do_register(device_id)
			.await
			.map(|(tokens, cookie)| {
				Response::new(proto::RegistryResponse {
					tokens: Some(tokens),
					cookie: cookie.0.to_string(),
				})
			})
			.trace_err(format!("Cookie register with device_id {}", device_id))
			.map_err(|_| Status::internal(""))
	}

	async fn login(
		&self,
		request: Request<LoginRequest>,
	) -> Result<Response<proto::LoginResponse>, Status> {
		COOKIE_LOGIN_COUNTER.inc();
		let request = request.get_ref();
		let cookie = request.cookie.as_str();
		let uuid = Uuid::try_from(cookie)
			.trace_err(format!("Convert cookie to uuid {}", cookie))
			.map_err(|_| Status::internal(""))?;
		let result = self
			.do_login(request, Cookie::from(uuid))
			.await
			.map(|tokens| Response::new(proto::LoginResponse { tokens }))
			.trace_err(format!("Login by cookie {}", uuid))
			.map_err(|_| Status::internal(""))?;
		Ok(result)
	}
}

lazy_static! {
	static ref COOKIE_REGISTER_COUNTER: IntCounter = register_int_counter!(
		"cookie_user_register_count",
		"Count register user by cookie"
	)
	.unwrap();
	static ref COOKIE_LOGIN_COUNTER: IntCounter =
		register_int_counter!("cookie_user_login_count", "Count login user by cookie").unwrap();
}

#[cfg(test)]
mod test {
	use std::time::Duration;

	use tonic::Request;

	use crate::cookie::service::CookieService;
	use crate::proto::cookie_server::Cookie;
	use crate::proto::{LoginRequest, RegistryRequest};
	use crate::tokens::tests::{stub_token_service, PUBLIC_KEY};
	use crate::users::service::UserService;
	use crate::ydb::test::setup_ydb;

	#[tokio::test]
	async fn should_register_and_login() {
		let (ydb_client, _instance) = setup_ydb().await;
		let (_node, token_service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(100)).await;
		let service = CookieService::new(
			ydb_client.table_client(),
			token_service,
			UserService::new(ydb_client.table_client()),
		);
		let result = service
			.register(Request::new(RegistryRequest {
				device_id: "device".to_string(),
			}))
			.await;
		let register_response = result.unwrap();
		let register_response = register_response.get_ref();

		let jwt = jwt_tonic_user_uuid::JWTUserTokenParser::new(PUBLIC_KEY.to_string());
		let register_user_uuid = jwt
			.get_user_uuid(
				register_response
					.tokens
					.as_ref()
					.unwrap()
					.session
					.to_owned(),
			)
			.unwrap();

		let login_result = service
			.login(Request::new(LoginRequest {
				cookie: register_response.cookie.clone(),
				device_id: "some-device".to_string(),
			}))
			.await;
		let login_response = login_result.unwrap();
		let login_response = login_response.get_ref();

		let login_user_id = jwt
			.get_user_uuid(login_response.tokens.as_ref().unwrap().session.to_owned())
			.unwrap();

		assert_eq!(register_user_uuid, login_user_id);
	}

	#[tokio::test]
	async fn should_not_login_with_wrong_cookie() {
		let (ydb_client, _instance) = setup_ydb().await;
		let (_node, token_service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(100)).await;
		let service = CookieService::new(
			ydb_client.table_client(),
			token_service,
			UserService::new(ydb_client.table_client()),
		);
		let login_result = service
			.login(Request::new(LoginRequest {
				cookie: "88c56aca-7111-4c80-b49d-86ebb3d2f697".to_string(),
				device_id: "some-device".to_string(),
			}))
			.await;
		assert!(login_result.unwrap().get_ref().tokens.is_none());
	}
}
