use std::fmt::Debug;

use cheetah_libraries_microservice::trace::ResultErrorTracer;
use cheetah_libraries_ydb::converters::YDBValueConverter;
use cheetah_libraries_ydb::{query, select};
use uuid::Uuid;
use ydb::{TableClient, Value, YdbOrCustomerError};

use crate::ydb::table::{ToDbTable, COLUMN_FIELD_NAME, COLUMN_FIELD_VALUE, COLUMN_USER};
use crate::ydb::{primitive::Primitive, Error};

pub struct YDBFetch {
	client: TableClient,
}

impl YDBFetch {
	pub fn new(client: TableClient) -> Self {
		Self { client }
	}

	pub async fn get<T>(&self, user: &Uuid, field_name: &str) -> Result<T, Error>
	where
		T: Primitive + YDBValueConverter + ToDbTable,
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
		query_result
			.map(|v| v[0].clone())
			.trace_and_map_err(format!("Fetch operation failed"), |e| e.into())
	}
}
