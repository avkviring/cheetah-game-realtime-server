use sqlx::types::ipnetwork::IpNetwork;
use sqlx::PgPool;

use crate::users::UserId;

pub struct GoogleStorage {
	pool: PgPool,
}

impl GoogleStorage {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}

	pub async fn attach(&self, user: UserId, google_id: &str, ip: IpNetwork) {
		let mut tx = self.pool.begin().await.unwrap();

		sqlx::query("delete from google_users where user_id=$1 or google_id=$2")
			.bind(user)
			.bind(google_id)
			.execute(&mut tx)
			.await
			.unwrap();

		sqlx::query("insert into google_users values($1,$2, $3)")
			.bind(user)
			.bind(ip)
			.bind(google_id)
			.execute(&mut tx)
			.await
			.unwrap();

		sqlx::query("insert into google_users_history (ip, user_id, google_id) values($1,$2, $3)")
			.bind(ip)
			.bind(user)
			.bind(google_id)
			.execute(&mut tx)
			.await
			.unwrap();

		tx.commit().await.unwrap();
	}

	pub async fn find(&self, google_id: &str) -> Option<UserId> {
		sqlx::query_as("select user_id from google_users where google_id=$1")
			.bind(google_id)
			.fetch_optional(&self.pool)
			.await
			.unwrap()
	}
}

#[cfg(test)]
pub mod tests {
	use std::str::FromStr;

	use chrono::NaiveDateTime;
	use sqlx::types::ipnetwork::IpNetwork;
	use testcontainers::clients::Cli;

	use crate::postgresql::test::setup_postgresql_storage;
	use crate::users::{UserId, UserService};

	use super::GoogleStorage;

	#[tokio::test]
	pub async fn should_attach() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let user_service = UserService::new(pool.clone());
		let google_storage = GoogleStorage::new(pool.clone());

		let ip = IpNetwork::from_str("127.0.0.1").unwrap();

		let user_a = user_service.create(ip).await.into();
		let user_b = user_service.create(ip).await.into();
		google_storage.attach(user_a, "a@kviring.com", ip).await;
		google_storage.attach(user_b, "b@kviring.com", ip).await;

		assert_eq!(google_storage.find("a@kviring.com").await.unwrap(), user_a);
		assert_eq!(google_storage.find("b@kviring.com").await.unwrap(), user_b);
		assert!(google_storage.find("c@kviring.com").await.is_none());
	}

	#[tokio::test]
	pub async fn should_history() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let user_service = UserService::new(pool.clone());
		let google_storage = GoogleStorage::new(pool.clone());

		let ip = IpNetwork::from_str("127.0.0.1").unwrap();

		let user = user_service.create(ip).await.into();
		google_storage.attach(user, "a@kviring.com", ip).await;
		google_storage.attach(user, "b@kviring.com", ip).await;

		let result: Vec<(NaiveDateTime, UserId, String)> =
			sqlx::query_as("select time, user_id, google_id from google_users_history order by time")
				.fetch_all(&pool)
				.await
				.unwrap();

		let i1 = result.get(0).unwrap();
		assert_eq!(i1.1, user);
		assert_eq!(i1.2, "a@kviring.com".to_owned());

		let i2 = result.get(1).unwrap();
		assert_eq!(i2.1, user);
		assert_eq!(i2.2, "b@kviring.com".to_owned());
	}

	/// Перепривязка google_id от одного пользователя к другому
	#[tokio::test]
	pub async fn should_reattach_1() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let user_service = UserService::new(pool.clone());
		let google_storage = GoogleStorage::new(pool.clone());

		let ip = IpNetwork::from_str("127.0.0.1").unwrap();

		let user_a = user_service.create(ip).await.into();
		let user_b = user_service.create(ip).await.into();
		let user_c = user_service.create(ip).await.into();

		google_storage.attach(user_a, "a@kviring.com", ip).await;
		google_storage.attach(user_b, "a@kviring.com", ip).await;
		google_storage.attach(user_c, "c@kviring.com", ip).await;

		assert_eq!(google_storage.find("a@kviring.com").await.unwrap(), user_b);
		// проверяем что данные других пользователей не изменились
		assert_eq!(google_storage.find("c@kviring.com").await.unwrap(), user_c);
	}

	/// Перепривязка google_id для пользователя
	#[tokio::test]
	pub async fn should_reattach_2() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let user_service = UserService::new(pool.clone());
		let google_storage = GoogleStorage::new(pool.clone());

		let ip = IpNetwork::from_str("127.0.0.1").unwrap();

		let user_a = user_service.create(ip).await.into();
		google_storage.attach(user_a, "a@kviring.com", ip).await;
		google_storage.attach(user_a, "aa@kviring.com", ip).await;

		let user_b = user_service.create(ip).await.into();
		google_storage.attach(user_b, "c@kviring.com", ip).await;

		assert!(google_storage.find("a@kviring.com").await.is_none());
		assert_eq!(google_storage.find("aa@kviring.com").await.unwrap(), user_a);

		// проверяем что данные другого пользователя не удалены
		assert_eq!(google_storage.find("c@kviring.com").await.unwrap(), user_b);
	}
}
