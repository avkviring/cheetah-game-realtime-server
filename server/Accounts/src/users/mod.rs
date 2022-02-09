use serde::{Deserialize, Serialize};
pub use sqlx::{types::ipnetwork::IpNetwork, PgPool};

#[derive(Clone)]
pub struct UserService {
	pool: PgPool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::FromRow, sqlx::Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct UserId(i64);

impl From<UserId> for i64 {
	fn from(id: UserId) -> Self {
		id.0
	}
}

impl From<UserId> for u64 {
	fn from(id: UserId) -> Self {
		id.0 as u64
	}
}

impl From<i64> for UserId {
	fn from(id: i64) -> Self {
		Self(id)
	}
}

impl From<u64> for UserId {
	fn from(id: u64) -> Self {
		Self(id as i64)
	}
}

impl UserService {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}

	/// Создать игрока, для каждого игры создается запись вида `id, create_date`
	/// Все остальное храниться в отдельных таблицах
	pub async fn create(&self, ip: IpNetwork) -> UserId {
		sqlx::query_as("insert into users (ip) values ($1) returning id")
			.bind(ip)
			.fetch_one(&self.pool)
			.await
			.unwrap()
	}
}

#[cfg(test)]
pub mod tests {
	use chrono::NaiveDateTime;
	use sqlx::types::ipnetwork::IpNetwork;
	use testcontainers::clients::Cli;

	use crate::postgresql::test::setup_postgresql_storage;
	use crate::users::UserId;

	use super::UserService;

	#[tokio::test]
	pub async fn should_create() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let users = UserService::new(pool.clone());

		let addr_a = "127.1.0.1".parse().unwrap();
		let id_a = users.create(addr_a).await;
		assert_eq!(id_a, UserId(1));

		let addr_b = "127.1.0.1".parse().unwrap();
		let id_b = users.create(addr_b).await;
		assert_eq!(id_b, UserId(2));

		let result: Vec<(UserId, IpNetwork, NaiveDateTime)> = sqlx::query_as("select id, ip, create_time from users")
			.fetch_all(&pool)
			.await
			.unwrap();

		assert_eq!(result[0].0, id_a);
		assert_eq!(result[0].1, addr_a);

		assert_eq!(result[1].0, id_b);
		assert_eq!(result[1].1, addr_b);
	}
}
