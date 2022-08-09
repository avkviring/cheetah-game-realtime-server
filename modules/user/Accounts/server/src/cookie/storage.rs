use sqlx::{PgPool, Row};
use thiserror::Error;
use uuid::Uuid;

use crate::cookie::Cookie;
use crate::users::user::User;

pub struct CookieStorage {
	pg_pool: PgPool,
}

impl CookieStorage {
	pub fn new(pg_pool: PgPool) -> Self {
		Self { pg_pool }
	}

	pub async fn attach(&self, user: User) -> Result<Cookie, sqlx::Error> {
		let cookie = Cookie(Uuid::new_v4());
		sqlx::query("insert into cheetah_user_accounts_cookie (cookie,user_uuid) values($1, $2)")
			.bind(cookie.0)
			.bind(user.0)
			.execute(&self.pg_pool)
			.await?;
		Ok(cookie)
	}

	pub async fn find(&self, cookie: &Cookie) -> Result<Option<User>, sqlx::Error> {
		Ok(
			sqlx::query("select user_uuid from cheetah_user_accounts_cookie where cookie=$1")
				.bind(cookie.0)
				.fetch_optional(&self.pg_pool)
				.await?
				.map(|r| User(r.get(0))),
		)
	}
}

#[cfg(test)]
pub mod tests {
	use crate::cookie::storage::CookieStorage;
	use crate::cookie::Cookie;
	use crate::postgres::test::setup_postgresql;
	use crate::users::service::UserService;

	#[tokio::test]
	pub async fn should_attach() {
		let (pg_pool, _container) = setup_postgresql().await;
		let user_service = UserService::new(pg_pool.clone());
		let cookie_storage = CookieStorage::new(pg_pool);

		let user_a = user_service.create().await.unwrap();
		let user_b = user_service.create().await.unwrap();

		let cookie_a = cookie_storage.attach(user_a).await.unwrap();
		let cookie_b = cookie_storage.attach(user_b).await.unwrap();

		assert_eq!(
			cookie_storage.find(&cookie_a).await.unwrap().unwrap(),
			user_a
		);
		assert_eq!(
			cookie_storage.find(&cookie_b).await.unwrap().unwrap(),
			user_b
		);
	}
}
