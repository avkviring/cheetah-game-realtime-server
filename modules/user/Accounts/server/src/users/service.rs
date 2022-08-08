use sqlx::PgPool;

use crate::users::user::User;

#[derive(Clone)]
pub struct UserService {
	pg_pool: PgPool,
}

impl UserService {
	pub fn new(pg_pool: PgPool) -> Self {
		Self { pg_pool }
	}

	pub async fn create(&self) -> Result<User, sqlx::Error> {
		let user = User::default();
		sqlx::query("insert into cheetah_user_account_users(user_uuid) values($1)")
			.bind(user.0)
			.execute(&self.pg_pool)
			.await?;
		Ok(user)
	}
}

#[cfg(test)]
pub mod tests {
	use std::time::Duration;

	use sqlx::postgres::PgRow;
	use sqlx::Row;
	use uuid::Uuid;

	use crate::postgres::test::setup_postgresql;

	use super::UserService;

	#[tokio::test]
	pub async fn should_create() {
		let (pg_pool, _instance) = setup_postgresql().await;
		let users = UserService::new(pg_pool.clone());

		let user_a = users.create().await.unwrap();
		let user_b = users.create().await.unwrap();

		let users: Vec<Uuid> = sqlx::query("select user_uuid from cheetah_user_account_users")
			.fetch_all(&pg_pool)
			.await
			.unwrap()
			.iter_mut()
			.map(|r| r.get(0))
			.collect();

		assert!(users.iter().any(|i| *i == user_a.0));
		assert!(users.iter().any(|i| *i == user_b.0));
	}
}
