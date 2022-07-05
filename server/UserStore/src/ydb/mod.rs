mod fetch;
mod numeric;
mod primitive;
mod table;
mod update;

pub use fetch::YDBFetch;
use include_dir::{include_dir, Dir};
pub use update::YDBUpdate;
use ydb::{YdbError, YdbOrCustomerError};

#[allow(dead_code)]
static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

#[allow(dead_code)]
pub const DB_NAME: &str = "userstore";

#[derive(Debug)]
pub enum Error {
	NoSuchField,
	DatabaseError(YdbOrCustomerError),
}

impl From<YdbOrCustomerError> for Error {
	fn from(e: YdbOrCustomerError) -> Self {
		match e {
			YdbOrCustomerError::YDB(YdbError::NoRows) => Error::NoSuchField,
			_ => Error::DatabaseError(e),
		}
	}
}

#[cfg(test)]
mod test {
	use std::sync::Arc;

	use crate::ydb::MIGRATIONS_DIR;
	use cheetah_libraries_ydb::migration::Migrator;
	use cheetah_libraries_ydb::test_container as ydb_test;
	use cheetah_libraries_ydb::test_container::YDBTestInstance;
	use uuid::Uuid;
	use ydb::Client;

	use crate::ydb::{YDBFetch, YDBUpdate, DB_NAME};

	#[tokio::test]
	async fn test_get_double() {
		let (_instance, client) = setup_ydb().await;

		let user = Uuid::new_v4();
		let field_name = "cringebar";
		let expected_value = 666.666;

		let update = YDBUpdate::new(client.table_client());
		update
			.set(&user, field_name, &expected_value)
			.await
			.unwrap();

		let fetch = YDBFetch::new(client.table_client());
		let actual_value: f64 = fetch.get(&user, field_name).await.unwrap();

		assert_eq!(expected_value, actual_value);
	}

	#[tokio::test]
	async fn test_get_string() {
		let (_instance, client) = setup_ydb().await;

		let user = Uuid::new_v4();
		let field_name = "displayname";
		let expected_value = "Potet";

		let update = YDBUpdate::new(client.table_client());
		update
			.set(&user, field_name, &expected_value)
			.await
			.unwrap();

		let fetch = YDBFetch::new(client.table_client());
		let actual_value: String = fetch.get(&user, field_name).await.unwrap();

		assert_eq!(expected_value, actual_value);
	}

	async fn setup_ydb() -> (Arc<YDBTestInstance>, Client) {
		let (_instance, client) = ydb_test::get_or_create_ydb_instance(DB_NAME).await;

		let mut m = Migrator::new_from_dir(&MIGRATIONS_DIR);
		m.migrate(&client).await.unwrap();

		(_instance, client)
	}
}
