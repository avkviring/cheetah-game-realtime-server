use std::fmt::Debug;

use cheetah_libraries_ydb::{query, select};
use uuid::Uuid;
use ydb::{TableClient, Value, YdbOrCustomerError};

use crate::ydb::table::{COLUMN_FIELD_NAME, COLUMN_FIELD_VALUE, COLUMN_USER};
use crate::ydb::{primitive::PrimitiveValue, Error};

pub struct Fetch {
	client: TableClient,
}

impl Fetch {
	pub fn new(client: TableClient) -> Self {
		Self { client }
	}

	pub async fn get<T>(&self, user: &Uuid, field_name: &str) -> Result<T, Error>
	where
		T: PrimitiveValue,
		Option<T>: TryFrom<Value>,
		<Option<T> as TryFrom<Value>>::Error: Debug,
	{
		let q = self.query(T::to_db_table());
		let q = q.as_str();

		let result: Result<Vec<T>, _> = select!(
			self.client,
			query!(q, user_uuid => user, field_name => field_name),
			value => T
		)
		.await;

		self.finalize(result)
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

	fn finalize<T: Clone>(
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

		let result = fetch.get::<i64>(&user, "missing").await;
		match result {
			Err(Error::FieldNotFound) => return,
			Err(other) => panic!("Expected Error::FieldNotFound, found {}", other),
			other => panic!("Expected error, found {:?}", other),
		}
	}
}
