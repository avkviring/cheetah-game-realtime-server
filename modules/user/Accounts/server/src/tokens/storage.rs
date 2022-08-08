use std::ops::Add;
use std::time::Duration;

use uuid::Uuid;

use crate::users::user::User;

///
///
///
#[derive(Clone)]
pub struct TokenStorage {
	ydb_table_client: TableClient,
	///
	/// Время жизни данных для пользователя
	///
	ttl: Duration,
}
impl TokenStorage {
	pub fn new(ydb_table_client: TableClient, ttl: Duration) -> Self {
		Self {
			ydb_table_client,
			ttl,
		}
	}

	pub async fn create_new_linked_uuid(
		&self,
		user: &User,
		device: &str,
		create_at: &Duration,
	) -> Result<Uuid, YdbOrCustomerError> {
		let token_uuid = Uuid::new_v4();
		update!(
			self.ydb_table_client,
			query!(
				"upsert into tokens(user, device, token_uuid, create_at) values ($user,$device,$token_uuid,$create_at)",
				user=>user,
				device=>device,
				create_at=>create_at,
				token_uuid=>token_uuid
			)
		)
		.await?;
		Ok(token_uuid)
	}

	pub(crate) async fn is_linked(
		&self,
		user: &User,
		device: &str,
		token_uuid: &Uuid,
		now: Duration,
	) -> Result<bool, YdbOrCustomerError> {
		self.ydb_table_client
			.retry_transaction(|mut t| async move {
				let query = query!(
					"select create_at from tokens where user=$user and device=$device and token_uuid=$token_uuid",
					user=>user,
					device=>device,
					token_uuid=>token_uuid
				);
				let result = t
					.query(query)
					.await?
					.into_only_result()
					.unwrap()
					.rows()
					.any(|mut row| {
						let time: Option<Duration> = row
							.remove_field_by_name("create_at")
							.unwrap()
							.try_into()
							.unwrap();
						time.unwrap().add(self.ttl) >= now
					});
				t.query(query!(
					"delete from tokens where user=$user and device=$device and token_uuid=$token_uuid",
					user=>user,
					device=>device,
					token_uuid=>token_uuid
				))
				.await?;
				t.commit().await?;

				Ok(result)
			})
			.await
	}
}

#[cfg(test)]
pub mod tests {
	use std::sync::Arc;
	use std::time::{Duration, SystemTime, UNIX_EPOCH};

	use uuid::Uuid;

	use ydb_steroids::test_container::YDBTestInstance;

	use crate::postgres::test::setup_ydb;
	use crate::tokens::storage::TokenStorage;
	use crate::users::user::User;

	#[tokio::test]
	async fn should_create_different_uuid() {
		let (storage, _instance) = setup().await;
		let user = User::default();
		let device = "device";
		let now = Duration::default();
		let uuid_1 = storage
			.create_new_linked_uuid(&user, device, &now)
			.await
			.unwrap();
		let uuid_2 = storage
			.create_new_linked_uuid(&user, device, &now)
			.await
			.unwrap();

		assert_ne!(uuid_1, uuid_2);
	}

	#[tokio::test]
	async fn should_is_linked_return_true_when_linked() {
		let (storage, _instance) = setup().await;

		let user = User::default();
		let device = "device".to_owned();
		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
		let version_1 = storage
			.create_new_linked_uuid(&user, &device, &now)
			.await
			.unwrap();
		let linked = storage
			.is_linked(&user, &device, &version_1, now)
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
			.is_linked(&user, &device, &Uuid::new_v4(), Duration::default())
			.await
			.unwrap();

		assert!(!linked);
	}

	#[tokio::test]
	async fn should_is_linked_return_false_when_expired() {
		let (storage, _instance) = setup().await;
		let user = User::default();
		let device = "device".to_owned();

		let now = Duration::from_secs(0);
		let uuid = storage
			.create_new_linked_uuid(&user, &device, &now)
			.await
			.unwrap();

		let now = Duration::from_secs(100);
		let linked = storage.is_linked(&user, &device, &uuid, now).await.unwrap();
		assert!(!linked)
	}

	async fn setup() -> (TokenStorage, Arc<YDBTestInstance>) {
		let (ydb, instance) = setup_ydb().await;
		let storage = TokenStorage::new(ydb.table_client(), Duration::from_secs(10));
		(storage, instance)
	}
}
