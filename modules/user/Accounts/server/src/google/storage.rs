use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use crate::users::user::User;

pub struct GoogleStorage {
	pg_pool: PgPool,
}

impl GoogleStorage {
	pub fn new(pg_pool: PgPool) -> Self {
		Self { pg_pool }
	}

	pub async fn attach(&self, user: User, google_id: &str) -> Result<(), sqlx::Error> {
		sqlx::query("delete from cheetah_user_accounts_google where user_uuid=$1 or google_id=$2")
			.bind(user.0)
			.bind(google_id)
			.execute(&self.pg_pool)
			.await?;

		sqlx::query("insert into cheetah_user_accounts_google (user_uuid, google_id) values($1, $2)")
			.bind(user.0)
			.bind(google_id)
			.execute(&self.pg_pool)
			.await?;

		sqlx::query("insert into cheetah_user_accounts_google_users_history (user_uuid, google_id) values($1,$2)")
			.bind(user.0)
			.bind(google_id)
			.execute(&self.pg_pool)
			.await?;

		Ok(())
	}

	pub async fn find(&self, google_id: &str) -> Result<Option<User>, sqlx::Error> {
		let result: Option<PgRow> = sqlx::query("select * from cheetah_user_accounts_google where google_id=$1")
			.bind(google_id)
			.fetch_optional(&self.pg_pool)
			.await?;
		Ok(result.map(|row| User(row.get(0))))
	}
}

#[cfg(test)]
pub mod tests {

	use sqlx::Row;

	use crate::postgres::test::setup_postgresql;
	use crate::users::service::UserService;
	use crate::users::user::User;

	use super::GoogleStorage;

	#[tokio::test]
	pub async fn should_attach() {
		let (pg_pool, _instance) = setup_postgresql().await;
		let user_service = UserService::new(pg_pool.clone());
		let google_storage = GoogleStorage::new(pg_pool);

		let user_a = user_service.create().await.unwrap();
		let user_b = user_service.create().await.unwrap();
		google_storage.attach(user_a, "a@kviring.com").await.unwrap();
		google_storage.attach(user_b, "b@kviring.com").await.unwrap();

		assert_eq!(google_storage.find("a@kviring.com").await.unwrap().unwrap(), user_a);
		assert_eq!(google_storage.find("b@kviring.com").await.unwrap().unwrap(), user_b);
		assert!(google_storage.find("c@kviring.com").await.unwrap().is_none());
	}

	#[tokio::test]
	pub async fn should_history() {
		let (pg_pool, _instance) = setup_postgresql().await;
		let user_service = UserService::new(pg_pool.clone());
		let google_storage = GoogleStorage::new(pg_pool.clone());

		let user = user_service.create().await.unwrap();
		google_storage.attach(user, "a@kviring.com").await.unwrap();
		google_storage.attach(user, "b@kviring.com").await.unwrap();

		let result: Vec<(User, String)> =
			sqlx::query("select user_uuid, google_id from cheetah_user_accounts_google_users_history order by created_at")
				.fetch_all(&pg_pool)
				.await
				.unwrap()
				.iter()
				.map(|row| (User(row.get(0)), row.get(1)))
				.collect();

		let i1 = result.get(0).unwrap();
		assert_eq!(i1.0, user);
		assert_eq!(i1.1, "a@kviring.com".to_owned());

		let i2 = result.get(1).unwrap();
		assert_eq!(i2.0, user);
		assert_eq!(i2.1, "b@kviring.com".to_owned());
	}

	/// Перепривязка google_id от одного пользователя к другому
	#[tokio::test]
	pub async fn should_reattach_1() {
		let (pg_pool, _instance) = setup_postgresql().await;
		let user_service = UserService::new(pg_pool.clone());
		let google_storage = GoogleStorage::new(pg_pool);

		let user_a = user_service.create().await.unwrap();
		let user_b = user_service.create().await.unwrap();
		let user_c = user_service.create().await.unwrap();

		google_storage.attach(user_a, "a@kviring.com").await.unwrap();
		google_storage.attach(user_b, "a@kviring.com").await.unwrap();
		google_storage.attach(user_c, "c@kviring.com").await.unwrap();

		assert_eq!(google_storage.find("a@kviring.com").await.unwrap().unwrap(), user_b);
		// проверяем что данные других пользователей не изменились
		assert_eq!(google_storage.find("c@kviring.com").await.unwrap().unwrap(), user_c);
	}

	/// Перепривязка google_id для пользователя
	#[tokio::test]
	pub async fn should_reattach_2() {
		let (pg_pool, _instance) = setup_postgresql().await;
		let user_service = UserService::new(pg_pool.clone());
		let google_storage = GoogleStorage::new(pg_pool);

		let user_a = user_service.create().await.unwrap();
		google_storage.attach(user_a, "a@kviring.com").await.unwrap();
		google_storage.attach(user_a, "aa@kviring.com").await.unwrap();

		let user_b = user_service.create().await.unwrap();
		google_storage.attach(user_b, "c@kviring.com").await.unwrap();

		assert!(google_storage.find("a@kviring.com").await.unwrap().is_none());
		assert_eq!(google_storage.find("aa@kviring.com").await.unwrap().unwrap(), user_a);

		// проверяем что данные другого пользователя не удалены
		assert_eq!(google_storage.find("c@kviring.com").await.unwrap().unwrap(), user_b);
	}
}
