use std::fmt::Debug;

use uuid::Uuid;
use ydb::{TableClient, Value, YdbOrCustomerError};
use ydb_steroids::{query, select};

use crate::ydb::{
	table::{
		COLUMN_FIELD_NAME, COLUMN_FIELD_VALUE, COLUMN_USER, DOUBLE_TABLE, LONG_TABLE, STRING_TABLE,
	},
	Error,
};

pub struct Fetch {
	client: TableClient,
}

impl Fetch {
	pub fn new(client: TableClient) -> Self {
		Self { client }
	}

	async fn get<T>(&self, user: &Uuid, field_name: &str, table_name: &str) -> Result<T, Error>
	where
		T: Clone,
		Option<T>: TryFrom<Value>,
		<Option<T> as TryFrom<Value>>::Error: Debug,
	{
		let q = self.query(table_name);
		let q = q.as_str();

		let result: Result<Vec<T>, _> = select!(
			self.client,
			query!(q, user_uuid => user, field_name => field_name),
			value => T
		)
		.await;

		self.final_result(result)
	}

	pub async fn get_double(&self, user: &Uuid, field_name: &str) -> Result<f64, Error> {
		self.get::<f64>(user, field_name, DOUBLE_TABLE).await
	}

	pub async fn get_long(&self, user: &Uuid, field_name: &str) -> Result<i64, Error> {
		self.get::<i64>(user, field_name, LONG_TABLE).await
	}

	pub async fn get_string(&self, user: &Uuid, field_name: &str) -> Result<String, Error> {
		self.get::<String>(user, field_name, STRING_TABLE).await
	}

	fn query(&self, table: &str) -> String {
		format!(
			"select {} from {} where {} = ${} and {} = ${}",
			COLUMN_FIELD_VALUE,
			table,
			COLUMN_USER,
			COLUMN_USER,
			COLUMN_FIELD_NAME,
			COLUMN_FIELD_NAME
		)
	}

	fn final_result<T: Clone>(
		&self,
		query_result: Result<Vec<T>, YdbOrCustomerError>,
	) -> Result<T, Error> {
		match query_result {
			Ok(v) => {
				if v.is_empty() {
					Err(Error::FieldNotFound)
				} else {
					Ok(v[0].clone())
				}
			}
			Err(e) => Err(e.into()),
		}
	}
}

#[cfg(test)]
mod test {
	use uuid::Uuid;

	use super::Fetch;
	use crate::ydb::{test::ydb_instance, Error};

	#[tokio::test]
	async fn test_fetch_missing_field_fails() {
		let (_instance, client) = ydb_instance("test_fetch_missing_field_fails").await;

		let fetch = Fetch::new(client.table_client());
		let user = Uuid::new_v4();

		let result = fetch.get_long(&user, "missing").await;
		assert!(matches!(result, Err(Error::FieldNotFound)));
	}
}
