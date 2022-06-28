use crate::ydb::{FIELD_NAME, FIELD_VALUE, LONG_TABLE, USER};
use cheetah_libraries_ydb::{query, update};
use uuid::Uuid;
use ydb::{TableClient, YdbOrCustomerError};
pub struct YDBUpdate {
	client: TableClient,
}

impl YDBUpdate {
	pub fn new(client: TableClient) -> Self {
		Self { client }
	}

	pub async fn set_int(
		&self,
		user: &Uuid,
		field_name: &str,
		field_value: i64,
	) -> Result<(), YdbOrCustomerError> {
		let query = format!(
			"upsert into {} ({}, {}, {}) values (${}, ${}, ${})",
			LONG_TABLE, USER, FIELD_NAME, FIELD_VALUE, USER, FIELD_NAME, FIELD_VALUE
		);

		let q = query.as_str();
		let _ = update!(
			self.client,
			query!(q, user_uuid => user, field_name => field_name, value => field_value)
		)
		.await?;

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use crate::ydb::LONG_TABLE;

	use super::YDBUpdate;
	use cheetah_libraries_ydb::migration::Migrator;
	use cheetah_libraries_ydb::test_container as ydb_test;
	use cheetah_libraries_ydb::{query, select};
	use include_dir::include_dir;
	use uuid::Uuid;

	#[tokio::test]
	async fn test_set_int() {
		let (_instance, client) = ydb_test::get_or_create_ydb_instance("userstore").await;
		let update = YDBUpdate::new(client.table_client());
		let mut m = Migrator::new_from_dir(&include_dir!("$CARGO_MANIFEST_DIR/migrations"));
		m.migrate(&client).await.unwrap();

		let user_id = Uuid::new_v4();
		update
			.set_int(&user_id, "points".into(), 666)
			.await
			.unwrap();

		let tc = client.table_client();
		let res: Vec<i64> =
			select!(tc, query!(format!("select value from {}", LONG_TABLE)), value => i64)
				.await
				.unwrap();

		assert_eq!(res.len(), 1);
		assert_eq!(res[0], 666);
	}
}
