use sqlx::{PgPool, Postgres, Row};
use uuid::Uuid;

use crate::storage::{TableName, COLUMN_FIELD_NAME, COLUMN_FIELD_VALUE, COLUMN_USER};

pub struct Fetcher {
	pg_pool: PgPool,
}

impl Fetcher {
	pub fn new(pg_pool: PgPool) -> Self {
		Self { pg_pool }
	}

	pub(crate) async fn get<T>(
		&self,
		user_uuid: &Uuid,
		field_name: &str,
	) -> Result<Option<T>, sqlx::Error>
	where
		T: TableName + Send,
		T: for<'r> sqlx::Decode<'r, Postgres> + sqlx::Type<Postgres>,
	{
		let result: Option<T> = sqlx::query(self.create_select_query(T::table_name()).as_str())
			.bind(user_uuid)
			.bind(field_name)
			.fetch_optional(&self.pg_pool)
			.await?
			.map(|row| row.get(0));

		Ok(result)
	}

	fn create_select_query(&self, table: &str) -> String {
		format!(
			"select {field_value} from {table} where {user} = $1 and {field_name} = $2",
			table = table,
			field_value = COLUMN_FIELD_VALUE,
			user = COLUMN_USER,
			field_name = COLUMN_FIELD_NAME,
		)
	}
}

#[cfg(test)]
mod test {
	use uuid::Uuid;

	use crate::postgres::test::setup_postgresql;

	use super::Fetcher;

	#[tokio::test]
	async fn test_fetch_missing_field_fails() {
		let (pg_pool, _instance) = setup_postgresql().await;

		let fetch = Fetcher::new(pg_pool);
		let user = Uuid::new_v4();

		let result = fetch.get::<i64>(&user, "missing").await;
		assert!(result.is_ok());
		assert!(result.unwrap().is_none());
	}
}
