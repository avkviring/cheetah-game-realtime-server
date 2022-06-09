use uuid::Uuid;
use ydb::{Bytes, TableClient, YdbOrCustomerError};

use cheetah_libraries_ydb::{query, update};

use crate::users::user::User;

pub struct GoogleStorage {
	ydb_table_client: TableClient,
}

impl GoogleStorage {
	pub fn new(ydb_table_client: TableClient) -> Self {
		Self { ydb_table_client }
	}

	pub async fn attach(&self, user: User, google_id: &str) -> Result<(), YdbOrCustomerError> {
		update!(
			self.ydb_table_client,
			query!(
				"delete from google_users where user=$user or google_id=$google_id",
				user=>user,
				google_id=>google_id
			)
		)
		.await?;

		self.ydb_table_client
			.retry_transaction(|mut t| async move {
				let q = r#"
					insert into google_users (user, google_id) values($user, $google_id);
					insert into google_users_history (user, google_id, time) values($user,$google_id, CurrentUtcDatetime());
				"#;
				t.query(query!(q, user=>user, google_id=>google_id)).await?;
				t.commit().await?;
				Ok(())
			})
			.await
	}

	pub async fn find(&self, google_id: &str) -> Result<Option<User>, YdbOrCustomerError> {
		self.ydb_table_client
			.retry_transaction(|mut t| async move {
				let query_result = t
					.query(query!("select * from google_users where google_id=$google_id", 
						google_id=>google_id))
					.await?;
				match query_result.into_only_row() {
					Ok(mut row) => {
						let user_uuid: Option<Bytes> = row.remove_field_by_name("user")?.try_into()?;
						let user_uuid: Vec<u8> = user_uuid.unwrap().into();
						Ok(Some(User::from(Uuid::from_slice(user_uuid.as_slice()).unwrap())))
					}
					Err(_) => Ok(None),
				}
			})
			.await
	}
}

#[cfg(test)]
pub mod tests {
	use std::time::Duration;

	use uuid::Uuid;
	use ydb::{Bytes, Query};

	use crate::users::service::UserService;
	use crate::users::user::User;
	use crate::ydb::test::setup_ydb;

	use super::GoogleStorage;

	#[tokio::test]
	pub async fn should_attach() {
		let (ydb_client, _instance) = setup_ydb().await;
		let user_service = UserService::new(ydb_client.table_client());
		let google_storage = GoogleStorage::new(ydb_client.table_client());

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
		let (ydb_client, _instance) = setup_ydb().await;
		let user_service = UserService::new(ydb_client.table_client());
		let google_storage = GoogleStorage::new(ydb_client.table_client());

		let user = user_service.create().await.unwrap();
		google_storage.attach(user, "a@kviring.com").await.unwrap();
		google_storage.attach(user, "b@kviring.com").await.unwrap();

		let result: Vec<(Duration, User, String)> = ydb_client
			.table_client()
			.retry_transaction(|mut t| async move {
				Ok(t.query(Query::new(
					"select time, user, google_id from google_users_history order by time",
				))
				.await
				.unwrap()
				.into_only_result()
				.unwrap()
				.rows()
				.map(|mut row| {
					let duration: Option<Duration> = row.remove_field_by_name("time").unwrap().try_into().unwrap();
					let user_uuid: Option<Bytes> = row.remove_field_by_name("user").unwrap().try_into().unwrap();
					let google_id: Option<String> = row.remove_field_by_name("google_id").unwrap().try_into().unwrap();
					let user_uuid: Vec<u8> = user_uuid.unwrap().into();
					(
						duration.unwrap(),
						Uuid::from_slice(user_uuid.as_slice()).unwrap().into(),
						google_id.unwrap(),
					)
				})
				.collect())
			})
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
		let (ydb_client, _instance) = setup_ydb().await;
		let user_service = UserService::new(ydb_client.table_client());
		let google_storage = GoogleStorage::new(ydb_client.table_client());

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
		let (ydb_client, _instance) = setup_ydb().await;
		let user_service = UserService::new(ydb_client.table_client());
		let google_storage = GoogleStorage::new(ydb_client.table_client());

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
