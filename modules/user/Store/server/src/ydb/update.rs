use std::ops::Add;

use uuid::Uuid;
use ydb::TableClient;
use ydb_steroids::converters::YDBValueConverter;
use ydb_steroids::{query, update};

use crate::ydb::table::{
	ydb_type_to_table_name, COLUMN_FIELD_NAME, COLUMN_FIELD_VALUE, COLUMN_USER,
};
use crate::ydb::Error;

pub struct Update {
	client: TableClient,
}

impl Update {
	pub fn new(client: TableClient) -> Self {
		Self { client }
	}

	pub async fn set<T: YDBValueConverter>(
		&self,
		user: &Uuid,
		field_name: &str,
		value: &T,
	) -> Result<(), Error> {
		let table = ydb_type_to_table_name(value.get_type_name());
		self.update(user, field_name, value, &self.upsert_query(table))
			.await
	}

	pub async fn increment<T: YDBValueConverter + Add>(
		&self,
		user: &Uuid,
		field_name: &str,
		value: &T,
	) -> Result<(), Error> {
		let table = ydb_type_to_table_name(value.get_type_name());
		self.update(user, field_name, value, &self.increment_query(table))
			.await
	}

	async fn update<T: YDBValueConverter>(
		&self,
		user: &Uuid,
		field_name: &str,
		value: &T,
		query: &str,
	) -> Result<(), Error> {
		update!(
			self.client,
			query!(query, user_uuid => user, field_name => field_name, value => value, increment => value)
		)
		.await
		.map_err(|e| e.into())
	}

	fn upsert_query(&self, table: &str) -> String {
		format!(
			"upsert into {} ({}, {}, {}) values (${}, ${}, ${})",
			table,
			COLUMN_USER,
			COLUMN_FIELD_NAME,
			COLUMN_FIELD_VALUE,
			COLUMN_USER,
			COLUMN_FIELD_NAME,
			COLUMN_FIELD_VALUE
		)
	}

	fn increment_query(&self, table: &str) -> String {
		format!(
			"update {} set {} = {} + ${} where {} = ${} and {} = ${}",
			table,
			COLUMN_FIELD_VALUE,
			COLUMN_FIELD_VALUE,
			"increment",
			COLUMN_USER,
			COLUMN_USER,
			COLUMN_FIELD_NAME,
			COLUMN_FIELD_NAME,
		)
	}
}

#[cfg(test)]
mod test {

	use super::Update;
	use uuid::Uuid;
	use ydb_steroids::{query, select};

	use crate::ydb::table::LONG_TABLE;
	use crate::ydb::test::ydb_instance;

	#[tokio::test]
	async fn test_set_long() {
		let (_instance, client) = ydb_instance("test_set_long").await;

		let update = Update::new(client.table_client());
		let user_id = Uuid::new_v4();
		let value = 666;
		update.set(&user_id, "points", &value).await.unwrap();

		let res: Vec<i64> =
			select!(client.table_client(), query!(format!("select value from {}", LONG_TABLE)), value => i64)
				.await
				.unwrap();

		assert_eq!(res.len(), 1);
		assert_eq!(res[0], value);
	}

	#[tokio::test]
	async fn test_increment() {
		let (_instance, client) = ydb_instance("test_increment").await;

		let update = Update::new(client.table_client());
		let user = Uuid::new_v4();
		let field_name = "incrementable";

		update.set::<i64>(&user, field_name, &128).await.unwrap();

		update
			.increment::<i64>(&user, field_name, &-55)
			.await
			.unwrap();

		let res: Vec<i64> =
			select!(client.table_client(), query!(format!("select value from {}", LONG_TABLE)), value => i64)
				.await
				.unwrap();

		assert_eq!(res.len(), 1);
		assert_eq!(res[0], 73);
	}

	#[tokio::test]
	async fn test_increment_missing_field() {
		let (_instance, client) = ydb_instance("test_increment_missing_field").await;

		let update = Update::new(client.table_client());
		let user = Uuid::new_v4();
		let field_name = "missing";

		update.increment(&user, field_name, &487).await.unwrap();
	}
}
