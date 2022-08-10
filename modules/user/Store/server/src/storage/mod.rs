use std::fmt::Display;

use include_dir::{include_dir, Dir};
use ydb::{YdbError, YdbOrCustomerError};

pub use fetch::Fetch;
pub use update::Update;

mod fetch;
mod table;
mod update;

#[allow(dead_code)]
static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

#[allow(dead_code)]
pub const DB_NAME: &str = "userstore";

#[derive(Debug)]
pub enum Error {
	FieldNotFound,
	DatabaseError(YdbOrCustomerError),
}

impl Error {
	pub fn is_server_side(&self) -> bool {
		matches!(self, Error::DatabaseError(_))
	}
}

impl From<YdbOrCustomerError> for Error {
	fn from(e: YdbOrCustomerError) -> Self {
		match e {
			YdbOrCustomerError::YDB(YdbError::NoRows) => Error::FieldNotFound,
			_ => Error::DatabaseError(e),
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{:?}", self))
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Error::DatabaseError(e) => Some(e),
			_ => None,
		}
	}
}

#[cfg(test)]
mod test {
	use std::sync::Arc;

	use uuid::Uuid;
	use ydb::Client;
	use ydb_steroids::migration::Migrator;
	use ydb_steroids::test_container as ydb_test;
	use ydb_steroids::test_container::YDBTestInstance;

	use crate::ydb::MIGRATIONS_DIR;
	use crate::ydb::{Fetch, Update};

	#[tokio::test]
	async fn test_get_double() {
		let (_instance, client) = ydb_instance("test_get_double").await;

		let user = Uuid::new_v4();
		let field_name = "cringebar";
		let expected_value = 666.666;

		let update = Update::new(client.table_client());
		update
			.set(&user, field_name, &expected_value)
			.await
			.unwrap();

		let fetch = Fetch::new(client.table_client());
		let actual_value = fetch.get_double(&user, field_name).await.unwrap();

		assert_eq!(expected_value, actual_value);
	}

	#[tokio::test]
	async fn test_get_string() {
		let (_instance, client) = ydb_instance("test_get_string").await;

		let user = Uuid::new_v4();
		let field_name = "displayname";
		let expected_value = "Potet";

		let update = Update::new(client.table_client());
		update
			.set(&user, field_name, &expected_value)
			.await
			.unwrap();

		let fetch = Fetch::new(client.table_client());
		let actual_value = fetch.get_string(&user, field_name).await.unwrap();

		assert_eq!(expected_value, actual_value);
	}

	pub async fn ydb_instance(db_name: &str) -> (Arc<YDBTestInstance>, Client) {
		let (_instance, client) = ydb_test::get_or_create_ydb_instance(db_name).await;

		let mut m = Migrator::new_from_dir(&MIGRATIONS_DIR);
		m.migrate(&client).await.unwrap();

		(_instance, client)
	}
}
