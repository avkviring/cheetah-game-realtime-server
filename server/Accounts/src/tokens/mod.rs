use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use ydb::TableClient;

use cheetah_libraries_microservice::jwt::{JWTTokenParser, SessionTokenClaims};

use crate::tokens::storage::TokenStorage;
use crate::users::user::User;

pub mod grpc;
pub mod storage;

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
	exp: usize,
	user: User,
	device_id: String,
	uuid: Uuid,
}

#[derive(Debug)]
pub struct Tokens {
	pub session: String,
	pub refresh: String,
}

#[derive(Debug, Error)]
pub enum JWTTokensServiceError {
	#[error("InvalidSignature")]
	InvalidSignature,

	#[error("Expired")]
	Expired,

	#[error("InvalidId")]
	InvalidId,

	#[error("StorageError {0}")]
	StorageError(String),
}

#[derive(Clone)]
pub struct TokensService {
	session_exp: Duration,
	refresh_exp: Duration,
	private_key: String,
	public_key: String,
	storage: TokenStorage,
}

const HOUR_IN_SEC: u64 = 60 * 60;
const SESSION_EXP: Duration = Duration::from_secs(10 * HOUR_IN_SEC);
const REFRESH_EXP: Duration = Duration::from_secs(30 * 24 * HOUR_IN_SEC);

impl TokensService {
	pub async fn new(ydb_table: TableClient, private_key: String, public_key: String) -> Self {
		let storage = TokenStorage::new(ydb_table, REFRESH_EXP);
		Self {
			session_exp: SESSION_EXP,
			refresh_exp: REFRESH_EXP,
			private_key,
			public_key,
			storage,
		}
	}

	pub fn new_with_storage(
		private_key: String,
		public_key: String,
		session_exp: Duration,
		refresh_exp: Duration,
		storage: TokenStorage,
	) -> Self {
		Self {
			session_exp,
			refresh_exp,
			private_key,
			public_key,
			storage,
		}
	}

	pub async fn create(
		&self,
		user: User,
		device_id: &str,
	) -> Result<Tokens, JWTTokensServiceError> {
		Ok(Tokens {
			session: self.create_session_token(&user),
			refresh: self.create_refresh_token(user, device_id).await?,
		})
	}

	async fn create_refresh_token(
		&self,
		user: User,
		device_id: &str,
	) -> Result<String, JWTTokensServiceError> {
		let now = TokensService::now();

		let uuid = self
			.storage
			.create_new_linked_uuid(&user, device_id, &now)
			.await
			.map_err(|e| JWTTokensServiceError::StorageError(format!("{:?}", e)))?;

		let claims = RefreshTokenClaims {
			exp: (now.as_secs() + self.refresh_exp.as_secs()) as usize,
			user,
			device_id: device_id.to_owned(),
			uuid,
		};
		let token = encode(
			&Header::new(Algorithm::ES256),
			&claims,
			&EncodingKey::from_ec_pem(self.private_key.as_bytes()).unwrap(),
		)
		.unwrap();
		Result::Ok(TokensService::remove_head(token))
	}

	fn create_session_token(&self, user: &User) -> String {
		let timestamp = TokensService::now();
		let claims = SessionTokenClaims {
			exp: (timestamp.as_secs() + self.session_exp.as_secs()) as usize,
			user: user.0,
		};

		let token = encode(
			&Header::new(Algorithm::ES256),
			&claims,
			&EncodingKey::from_ec_pem(self.private_key.as_bytes()).unwrap(),
		)
		.unwrap();
		TokensService::remove_head(token)
	}

	fn now() -> Duration {
		SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("Time went backwards")
	}

	fn remove_head(token: String) -> String {
		let collect: Vec<_> = token.split('.').collect();
		format!("{}.{}", collect[1], collect[2])
	}

	pub async fn refresh(&self, refresh_token: String) -> Result<Tokens, JWTTokensServiceError> {
		let token = JWTTokenParser::add_head(refresh_token);
		let mut validation = Validation::new(Algorithm::ES256);
		validation.leeway = 0;
		match jsonwebtoken::decode::<RefreshTokenClaims>(
			token.as_str(),
			&DecodingKey::from_ec_pem(self.public_key.as_bytes()).unwrap(),
			&validation,
		) {
			Ok(token) => {
				let user = token.claims.user;
				let device_id = token.claims.device_id;
				match self
					.storage
					.is_linked(&user, &device_id, &token.claims.uuid, TokensService::now())
					.await
				{
					Ok(linked) => {
						if linked {
							Result::Ok(Tokens {
								session: self.create_session_token(&user),
								refresh: self.create_refresh_token(user, &device_id).await?,
							})
						} else {
							Result::Err(JWTTokensServiceError::InvalidId)
						}
					}
					Err(e) => Result::Err(JWTTokensServiceError::StorageError(format!("{}", e))),
				}
			}
			Err(error) => match error.kind() {
				ErrorKind::ExpiredSignature => Result::Err(JWTTokensServiceError::Expired),
				_ => Result::Err(JWTTokensServiceError::InvalidSignature),
			},
		}
	}
}

#[cfg(test)]
pub mod tests {
	use std::ops::Add;
	use std::sync::Arc;
	use std::thread;
	use std::time::Duration;

	use cheetah_libraries_microservice::jwt::{JWTTokenParser, SessionTokenError};
	use cheetah_libraries_ydb::test_container::YDBTestInstance;

	use crate::tokens::storage::TokenStorage;
	use crate::tokens::{JWTTokensServiceError, TokensService};
	use crate::users::user::User;
	use crate::ydb::test::setup_ydb;

	#[tokio::test]
	async fn session_token_should_correct() {
		let (_node, service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(1)).await;
		let user = User::default();
		let tokens = service.create(user, "some-device-id").await.unwrap();

		let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
		let user_id_from_token = User(parser.get_user_uuid(tokens.session).unwrap());

		assert_eq!(user, user_id_from_token)
	}

	#[tokio::test]
	async fn session_token_should_exp() {
		let (_node, service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(1)).await;
		let tokens = service
			.create(User::default(), "some-device-id")
			.await
			.unwrap();
		thread::sleep(Duration::from_secs(2));
		let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
		let user_id_from_token = parser.get_user_uuid(tokens.session);
		assert!(matches!(
			user_id_from_token,
			Result::Err(SessionTokenError::Expired)
		))
	}

	#[tokio::test]
	async fn session_token_should_fail_if_not_correct() {
		let (_node, service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(1)).await;
		let tokens = service
			.create(1u128.into(), "some-device-id")
			.await
			.unwrap();
		let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
		let user_id_from_token = parser.get_user_uuid(tokens.session.replace("ey", "e1"));
		assert!(matches!(
			user_id_from_token,
			Result::Err(SessionTokenError::InvalidSignature)
		))
	}

	pub const PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEVVHNXKxoUNkoX9hnOJpSz6K2KDfi
gxaSXu+FIpP32qvcDgZPZ01tjnGjOysyPxRoZaMu/d9rHi3ulbceoYwS+Q==
-----END PUBLIC KEY-----";

	pub const PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgcg7dsJWSz8f40cEv
BTeGSzANXGlEzutd9IIm6/inl0ahRANCAARVUc1crGhQ2Shf2Gc4mlLPorYoN+KD
FpJe74Uik/faq9wOBk9nTW2OcaM7KzI/FGhloy7932seLe6Vtx6hjBL5
-----END PRIVATE KEY-----";

	pub async fn stub_token_service<'a>(
		session_exp: Duration,
		refresh_exp: Duration,
	) -> (Arc<YDBTestInstance>, TokensService) {
		let (ydb, instance) = setup_ydb().await;
		let storage = TokenStorage::new(
			ydb.table_client(),
			refresh_exp.clone().add(Duration::from_secs(1)),
		);
		let service = TokensService::new_with_storage(
			PRIVATE_KEY.to_string(),
			PUBLIC_KEY.to_string(),
			session_exp,
			refresh_exp,
			storage,
		);
		(instance, service)
	}

	#[tokio::test]
	async fn should_refresh_token_different_for_players() {
		let (_node, service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(100)).await;
		let tokens_for_player_a = service
			.create(User::default(), "some-devicea-id")
			.await
			.unwrap();
		let tokens_for_player_b = service
			.create(User::default(), "some-deviceb-id")
			.await
			.unwrap();
		assert_ne!(tokens_for_player_a.refresh, tokens_for_player_b.refresh)
	}

	#[tokio::test]
	async fn should_refresh_token() {
		let (_node, service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(100)).await;

		let user = User::default();
		let tokens = service
			.create(user.clone(), "some-device-id")
			.await
			.unwrap();

		let new_tokens = service.refresh(tokens.refresh.clone()).await.unwrap();
		// проверяем что это действительно новые токены
		assert_ne!(tokens.session, new_tokens.session);
		assert_ne!(tokens.refresh, new_tokens.refresh);
		// проверяем работоспособность новых токенов
		let get_user_uuid =
			JWTTokenParser::new(PUBLIC_KEY.to_owned()).get_user_uuid(new_tokens.session);
		assert!(matches!(get_user_uuid, Result::Ok(uuid) if uuid==user.0));

		// проверяем что новый refresh токен валидный
		service.refresh(new_tokens.refresh.clone()).await.unwrap();
	}

	///
	/// Проверяем время жизни refresh токена
	///
	#[tokio::test]
	async fn should_refresh_token_exp() {
		let (_node, service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(1)).await;
		let tokens = service
			.create(User::default(), "some-device-id")
			.await
			.unwrap();
		thread::sleep(Duration::from_secs(2));
		let result = service.refresh(tokens.refresh).await;
		assert!(matches!(
			result,
			Result::Err(JWTTokensServiceError::Expired)
		));
	}

	///
	/// Проверяем реакцию на нарушение подписи
	///
	#[tokio::test]
	async fn should_refresh_token_fail() {
		let (_node, service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(1)).await;
		let tokens = service
			.create(User::default(), "some-device-id")
			.await
			.unwrap();
		assert!(matches!(
			service
				.refresh(tokens.refresh.replace("eyJleHA", "eyJleHB"))
				.await,
			Result::Err(JWTTokensServiceError::InvalidSignature)
		));
	}

	///
	/// Проверяем что refresh токен может быть использован один раз
	///
	#[tokio::test]
	async fn should_refresh_token_can_use_once() {
		let (_node, service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(1)).await;
		let tokens = service
			.create(User::default(), "some-device-id")
			.await
			.unwrap();
		service.refresh(tokens.refresh.clone()).await.unwrap();
		assert!(matches!(
			service.refresh(tokens.refresh).await,
			Result::Err(JWTTokensServiceError::InvalidId)
		));
	}

	///
	/// Проверяем что выдача нового refresh токена инвалидирует старые
	///
	#[tokio::test]
	async fn should_refresh_token_can_invalidate_tokens() {
		let (_node, service) =
			stub_token_service(Duration::from_secs(1), Duration::from_secs(1)).await;
		let tokens_a = service
			.create(1u128.into(), "some-device-id")
			.await
			.unwrap();
		let tokens_b = service
			.create(1u128.into(), "some-device-id")
			.await
			.unwrap();
		service.refresh(tokens_b.refresh.clone()).await.unwrap();
		assert!(matches!(
			service.refresh(tokens_a.refresh).await,
			Result::Err(JWTTokensServiceError::InvalidId)
		));
	}
}
