use lazy_static::lazy_static;
use prometheus::{register_int_counter, IntCounter};
use sqlx::PgPool;
use tonic::{Request, Response};

use crate::cookie::storage::{CookieStorage, FindResult};
use crate::proto::SessionAndRefreshTokens;
use crate::tokens::TokensService;
use crate::users::{UserId, UserService};
use crate::{get_client_ip, proto};

pub mod storage;

pub struct CookieGrpcService {
	storage: CookieStorage,
	token_service: TokensService,
	user_service: UserService,
}

impl CookieGrpcService {
	pub fn new(pg_pool: PgPool, token_service: TokensService, user_service: UserService) -> Self {
		Self {
			storage: CookieStorage::new(pg_pool),
			token_service,
			user_service,
		}
	}
}

#[tonic::async_trait]
impl proto::cookie_server::Cookie for CookieGrpcService {
	async fn register(
		&self,
		request: Request<proto::RegistryRequest>,
	) -> Result<Response<proto::RegistryResponse>, tonic::Status> {
		COOKIE_REGISTER_COUNTER.inc();
		let ip = get_client_ip(request.metadata());
		let user: UserId = self.user_service.create(ip).await;
		let cookie: String = self.storage.attach(user).await;
		match self.token_service.create(user, &request.get_ref().device_id).await {
			Ok(tokens) => {
				let tokens = Some(SessionAndRefreshTokens {
					session: tokens.session,
					refresh: tokens.refresh,
				});
				Ok(Response::new(proto::RegistryResponse { tokens, cookie }))
			}
			Err(e) => Err(tonic::Status::internal(format!("{:?}", e))),
		}
	}

	async fn login(&self, request: Request<proto::LoginRequest>) -> Result<Response<proto::LoginResponse>, tonic::Status> {
		COOKIE_LOGIN_COUNTER.inc();
		let request = request.get_ref();
		match self.storage.find(&request.cookie).await {
			FindResult::NotFound => Ok((None, proto::login_response::Status::NotFound as i32)),
			FindResult::Linked => Ok((None, proto::login_response::Status::Linked as i32)),
			FindResult::Player(user) => self.token_service.create(user, &request.device_id).await.map(|tokens| {
				(
					Some(SessionAndRefreshTokens {
						session: tokens.session,
						refresh: tokens.refresh,
					}),
					proto::login_response::Status::Ok as i32,
				)
			}),
		}
		.map(|(tokens, status)| proto::LoginResponse { tokens, status })
		.map(Response::new)
		.map_err(|e| tonic::Status::internal(format!("{:?}", e)))
	}
}

lazy_static! {
	static ref COOKIE_REGISTER_COUNTER: IntCounter =
		register_int_counter!("cookie_user_register_count", "Count register user by cookie").unwrap();
	static ref COOKIE_LOGIN_COUNTER: IntCounter =
		register_int_counter!("cookie_user_login_count", "Count login user by cookie").unwrap();
}

#[cfg(test)]
mod test {
	use testcontainers::clients::Cli;
	use tonic::Request;

	use crate::cookie::CookieGrpcService;
	use crate::postgresql::test::setup_postgresql_storage;
	use crate::proto;
	use crate::proto::cookie_server::Cookie;
	use crate::proto::{LoginRequest, RegistryRequest};
	use crate::tokens::tests::{stub_token_service, PUBLIC_KEY};
	use crate::users::UserService;

	#[tokio::test]
	async fn should_register_and_login() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let (_node, token_service) = stub_token_service(1, 100).await;
		let service = CookieGrpcService::new(pool.clone(), token_service, UserService::new(pool.clone()));
		let result = service
			.register(Request::new(RegistryRequest {
				device_id: "device".to_string(),
			}))
			.await;
		let register_response = result.unwrap();
		let register_response = register_response.get_ref();

		let jwt = cheetah_libraries_microservice::jwt::JWTTokenParser::new(PUBLIC_KEY.to_string());
		let register_user_id = jwt
			.get_user_id(register_response.tokens.as_ref().unwrap().session.to_owned())
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
			.get_user_id(login_response.tokens.as_ref().unwrap().session.to_owned())
			.unwrap();

		assert_eq!(register_user_id, login_user_id);
	}

	#[tokio::test]
	async fn should_not_login_with_wrong_cookie() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let (_node, token_service) = stub_token_service(1, 100).await;
		let service = CookieGrpcService::new(pool.clone(), token_service, UserService::new(pool.clone()));
		let login_result = service
			.login(Request::new(LoginRequest {
				cookie: "some-cookie".to_string(),
				device_id: "some-device".to_string(),
			}))
			.await;
		assert_eq!(
			login_result.unwrap().get_ref().status,
			proto::login_response::Status::NotFound as i32
		);
	}
}
