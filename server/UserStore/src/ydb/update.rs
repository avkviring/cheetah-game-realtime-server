use cheetah_libraries_microservice::trace::ResultErrorTracer;
use cheetah_libraries_ydb::converters::YDBValueConverter;
use cheetah_libraries_ydb::{query, update};
use uuid::Uuid;
use ydb::TableClient;

use crate::ydb::numeric::Num;
use crate::ydb::table::{ToDbTable, COLUMN_FIELD_NAME, COLUMN_FIELD_VALUE, COLUMN_USER};
use crate::ydb::{primitive::Primitive, Error};

pub struct YDBUpdate {
	client: TableClient,
}

impl YDBUpdate {
	pub fn new(client: TableClient) -> Self {
		Self { client }
	}

	pub async fn set<T: Primitive + ToDbTable + YDBValueConverter>(
		&self,
		user: &Uuid,
		field_name: &str,
		value: &T,
	) -> Result<(), Error> {
		let q = self.upsert_query(T::to_db_table());
		let q = q.as_str();
		update!(
			self.client,
			query!(q, user_uuid => user, field_name => field_name, value => value)
		)
		.await
		.trace_and_map_err("Set operation failed", |e| e.into())
	}

	pub async fn increment<T: Num + ToDbTable + YDBValueConverter>(
		&self,
		user: &Uuid,
		field_name: &str,
		value: &T,
	) -> Result<(), Error> {
		let q = self.increment_query(T::to_db_table());
		let q = q.as_str();
		update!(
			self.client,
			query!(q, user_uuid => user, field_name => field_name, increment => value)
		)
		.await
		.trace_and_map_err("Increment operation failed", |e| e.into())
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
	use std::sync::Arc;

	use crate::ydb::{table::LONG_TABLE, DB_NAME, MIGRATIONS_DIR};

	use super::YDBUpdate;
	use cheetah_libraries_ydb::migration::Migrator;
	use cheetah_libraries_ydb::test_container::{self as ydb_test, YDBTestInstance};
	use cheetah_libraries_ydb::{query, select};
	use uuid::Uuid;
	use ydb::Client;

	#[tokio::test]
	async fn test_set_long() {
		let (_instance, client) = setup_db().await;

		let update = YDBUpdate::new(client.table_client());
		let user_id = Uuid::new_v4();
		let value = 666;
		update.set(&user_id, "points".into(), &value).await.unwrap();

		let res: Vec<i64> =
			select!(client.table_client(), query!(format!("select value from {}", LONG_TABLE)), value => i64)
				.await
				.unwrap();

		assert_eq!(res.len(), 1);
		assert_eq!(res[0], value);
	}

	#[tokio::test]
	async fn test_increment() {
		let (_instance, client) = setup_db().await;

		let update = YDBUpdate::new(client.table_client());
		let user = Uuid::new_v4();
		let field_name = "incrementable";

		update.set(&user, field_name, &128).await.unwrap();

		update.increment(&user, field_name, &-55).await.unwrap();

		let res: Vec<i64> =
			select!(client.table_client(), query!(format!("select value from {}", LONG_TABLE)), value => i64)
				.await
				.unwrap();

		assert_eq!(res.len(), 1);
		assert_eq!(res[0], 73);
	}

	async fn setup_db() -> (Arc<YDBTestInstance>, Client) {
		let (instance, client) = ydb_test::get_or_create_ydb_instance(DB_NAME).await;
		let mut m = Migrator::new_from_dir(&MIGRATIONS_DIR);
		m.migrate(&client).await.unwrap();

		(instance, client)
	}
}
