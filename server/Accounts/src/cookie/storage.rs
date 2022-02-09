use sqlx::PgPool;

use crate::users::UserId;

#[derive(Debug)]
enum AttachError {
	UniqueViolation,
	Query(sqlx::Error),
}

pub enum FindResult {
	NotFound,
	Player(UserId),
	Linked,
}
pub struct CookieStorage {
	pool: PgPool,
}

impl CookieStorage {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}

	pub async fn attach(&self, user: UserId) -> String {
		use rand::distributions::Alphanumeric;
		use rand::rngs::OsRng;
		use rand::Rng;

		loop {
			let cookie: String = OsRng.sample_iter(&Alphanumeric).take(128).map(char::from).collect();

			match self.do_attach(user, &cookie).await {
				Ok(_) => break cookie,
				Err(AttachError::UniqueViolation) => continue,
				Err(err) => panic!("{:?}", err),
			}
		}
	}

	async fn do_attach(&self, user: UserId, cookie: &str) -> Result<(), AttachError> {
		let mut tx = self.pool.begin().await.unwrap();

		let result: Result<_, sqlx::Error> = sqlx::query("insert into cookie_users (user_id,cookie) values($1,$2)")
			.bind(user as UserId)
			.bind(cookie)
			.execute(&mut tx)
			.await;

		tx.commit().await.unwrap();

		result.map(|_| ()).map_err(|err| match err {
			sqlx::Error::Database(err) if matches!(err.code().as_deref(), Some("23505")) => AttachError::UniqueViolation,
			err => AttachError::Query(err),
		})
	}

	pub async fn find(&self, cookie: &str) -> FindResult {
		sqlx::query_as("select user_id, linked from cookie_users where cookie=$1")
			.bind(cookie)
			.fetch_optional(&self.pool)
			.await
			.unwrap()
			.map_or(FindResult::NotFound, |(user, linked)| {
				if linked {
					FindResult::Linked
				} else {
					FindResult::Player(user)
				}
			})
	}

	pub async fn link_cookie(&self, user: UserId) {
		sqlx::query("update cookie_users set linked=true where user_id=$1")
			.bind(user)
			.fetch_optional(&self.pool)
			.await
			.unwrap();
	}

	pub async fn _mark_cookie_as_linked(user: UserId, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) {
		sqlx::query("update cookie_users set linked=true where user_id=$1")
			.bind(user)
			.execute(tx)
			.await
			.unwrap();
	}
}

#[cfg(test)]
pub mod tests {
	use std::str::FromStr;

	use sqlx::types::ipnetwork::IpNetwork;
	use testcontainers::clients::Cli;

	use crate::cookie::storage::{AttachError, CookieStorage, FindResult};
	use crate::postgresql::test::setup_postgresql_storage;
	use crate::users::UserService;

	#[tokio::test]
	pub async fn should_attach() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let user_service = UserService::new(pool.clone());
		let cookie_storage = CookieStorage::new(pool.clone());

		let ip = IpNetwork::from_str("127.0.0.1").unwrap();

		let user_a = user_service.create(ip).await.into();
		let user_b = user_service.create(ip).await.into();

		let cookie_a = cookie_storage.attach(user_a).await;
		let cookie_b = cookie_storage.attach(user_b).await;

		assert!(matches!(cookie_storage.find( &cookie_a).await,
                FindResult::Player(user) if user == user_a));
		assert!(matches!(cookie_storage.find( &cookie_b).await,
            FindResult::Player(user) if user == user_b));
	}

	#[tokio::test]
	pub async fn should_linked() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let user_service = UserService::new(pool.clone());
		let cookie_storage = CookieStorage::new(pool.clone());

		let ip = IpNetwork::from_str("127.0.0.1").unwrap();

		let user = user_service.create(ip).await.into();
		let cookie = cookie_storage.attach(user).await;

		cookie_storage.link_cookie(user).await;

		assert!(matches!(cookie_storage.find(&cookie).await, FindResult::Linked));
	}

	#[tokio::test]
	pub async fn should_check_duplicate() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let user_service = UserService::new(pool.clone());
		let cookie_storage = CookieStorage::new(pool.clone());

		let ip = IpNetwork::from_str("127.0.0.1").unwrap();
		let user = user_service.create(ip).await.into();
		assert!(cookie_storage.do_attach(user, "cookie").await.is_ok());
		assert!(matches!(
			cookie_storage.do_attach(user, "cookie").await,
			Err(AttachError::UniqueViolation)
		));
	}
}
