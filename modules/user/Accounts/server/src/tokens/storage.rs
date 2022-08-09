
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{NaiveDateTime};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;


use crate::users::user::User;

///
///
///
#[derive(Clone)]
pub struct TokenStorage {
	pg_pool: PgPool,
	///
	/// Время жизни данных для пользователя
	///
	ttl: Duration,
}
impl TokenStorage {
	pub fn new(pg_pool: PgPool, ttl: Duration) -> Self {
		Self { pg_pool, ttl }
	}

	pub async fn create_new_linked_uuid(
		&self,
		user: &User,
		device: &str,
	) -> Result<Uuid, sqlx::Error> {
		let token_uuid = Uuid::new_v4();
		// удаляем старую привязку
		sqlx::query("delete from cheetah_user_accounts_tokens where user_uuid=$1 and device=$2")
			.bind(user.0)
			.bind(device)
			.execute(&self.pg_pool)
			.await?;

		sqlx::query("insert into cheetah_user_accounts_tokens (user_uuid, device, token) values ($1, $2,$3)")
			.bind(user.0)
			.bind(device)
			.bind(token_uuid)
			.execute(&self.pg_pool).await?;
		Ok(token_uuid)
	}

	pub(crate) async fn is_linked(
		&self,
		user: &User,
		device: &str,
		token_uuid: &Uuid,
		now: SystemTime,
	) -> Result<bool, sqlx::Error> {
		let result: Option<PgRow> = sqlx::query(
			"select create_at from cheetah_user_accounts_tokens where user_uuid=$1 and device=$2 and token=$3",
		)
		.bind(user.0)
		.bind(device)
		.bind(token_uuid)
		.fetch_optional(&self.pg_pool)
		.await?;

		Ok(match result {
			None => false,
			Some(row) => {
				let created_at: NaiveDateTime = row.get(0);
				created_at.timestamp_millis() as u128 + self.ttl.as_millis()
					> now.duration_since(UNIX_EPOCH).unwrap().as_millis()
			}
		})
	}
}

#[cfg(test)]
pub mod tests {
	use std::ops::Add;
	use std::time::{Duration, SystemTime, UNIX_EPOCH};

	
	use testcontainers::images::postgres::Postgres;
	use testcontainers::Container;
	use uuid::Uuid;

	use crate::postgres::test::setup_postgresql;
	use crate::tokens::storage::TokenStorage;
	
	use crate::users::user::User;

	#[tokio::test]
	async fn should_create_different_uuid() {
		let (storage, _instance) = setup().await;
		let user = User::default();
		let device = "device";
		let uuid_1 = storage.create_new_linked_uuid(&user, device).await.unwrap();
		let uuid_2 = storage.create_new_linked_uuid(&user, device).await.unwrap();

		assert_ne!(uuid_1, uuid_2);
	}

	#[tokio::test]
	async fn should_is_linked_return_true_when_linked() {
		let (storage, _instance) = setup().await;

		let user = User::default();
		let device = "device".to_owned();
		let _now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
		let version_1 = storage
			.create_new_linked_uuid(&user, &device)
			.await
			.unwrap();
		let linked = storage
			.is_linked(&user, &device, &version_1, SystemTime::now())
			.await
			.unwrap();

		assert!(linked);
	}

	#[tokio::test]
	async fn should_is_linked_return_false_when_not_linked() {
		let (storage, _instance) = setup().await;

		let user = User::default();
		let device = "device".to_owned();
		let linked = storage
			.is_linked(&user, &device, &Uuid::new_v4(), SystemTime::now())
			.await
			.unwrap();

		assert!(!linked);
	}

	#[tokio::test]
	async fn should_is_linked_return_false_when_expired() {
		let (storage, _instance) = setup().await;
		let user = User::default();
		let device = "device".to_owned();

		let uuid = storage
			.create_new_linked_uuid(&user, &device)
			.await
			.unwrap();

		let offset = Duration::from_secs(100);
		let linked = storage
			.is_linked(&user, &device, &uuid, SystemTime::now().add(offset))
			.await
			.unwrap();
		assert!(!linked)
	}

	async fn setup() -> (TokenStorage, Container<'static, Postgres>) {
		let (pg_pool, instance) = setup_postgresql().await;
		let storage = TokenStorage::new(pg_pool, Duration::from_secs(10));
		(storage, instance)
	}
}
