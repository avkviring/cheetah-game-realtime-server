use crate::users::user::User;
use ydb::{TableClient, YdbOrCustomerError};
use ydb_steroids::error::is_unique_violation_error;
use ydb_steroids::{query, update};

#[derive(Clone)]
pub struct UserService {
	ydb_table_client: TableClient,
}

impl UserService {
	pub fn new(ydb_table_client: TableClient) -> Self {
		Self { ydb_table_client }
	}

	pub async fn create(&self) -> Result<User, YdbOrCustomerError> {
		loop {
			let user = User::default();
			let result = update!(
				self.ydb_table_client,
				query!(
					"insert into users (user,create_date) values($user,CurrentUtcDatetime())",
					user=>user)
			)
			.await;
			match result {
				Ok(_) => return Ok(user),
				Err(err) => {
					if !is_unique_violation_error(&err) {
						return Err(err);
					}
				}
			}
		}
	}
}

#[cfg(test)]
pub mod tests {
	use std::time::Duration;

	use uuid::Uuid;
	use ydb::Query;

	use crate::ydb::test::setup_ydb;

	use super::UserService;

	#[tokio::test]
	pub async fn should_create() {
		let (ydb_client, _instance) = setup_ydb().await;
		let table_client = ydb_client.table_client();
		let users = UserService::new(ydb_client.table_client());

		let user_a = users.create().await.unwrap();
		let user_b = users.create().await.unwrap();

		let users: Vec<_> = table_client
			.retry_transaction(|mut t| async move {
				let users = t.query(Query::new("select * from users")).await?;
				Ok(users
					.into_only_result()
					.unwrap()
					.rows()
					.map(|mut row| {
						let uuid: Option<ydb::Bytes> = row
							.remove_field_by_name("user")
							.unwrap()
							.try_into()
							.unwrap();
						let uuid = Into::<Vec<u8>>::into(uuid.unwrap());
						let uuid = Uuid::from_slice(uuid.as_slice()).unwrap();

						let date: Option<Duration> = row
							.remove_field_by_name("create_date")
							.unwrap()
							.try_into()
							.unwrap();
						(uuid, date.unwrap())
					})
					.collect())
			})
			.await
			.unwrap();
		assert!(users.iter().any(|i| i.0 == user_a.0));
		assert!(users.iter().any(|i| i.0 == user_b.0));
	}
}
