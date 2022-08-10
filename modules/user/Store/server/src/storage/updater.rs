use std::ops::Add;

use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use crate::storage::{TableName, COLUMN_FIELD_NAME, COLUMN_FIELD_VALUE, COLUMN_USER};

pub struct Updater {
	pg_pool: PgPool,
}

impl Updater {
	pub fn new(pg_pool: PgPool) -> Self {
		Self { pg_pool }
	}

	pub async fn set<T>(&self, user: &Uuid, field_name: &str, value: T) -> Result<(), sqlx::Error>
	where
		T: TableName,
		T: for<'r> sqlx::Encode<'r, Postgres> + sqlx::Type<Postgres> + Send,
	{
		let table = T::table_name();
		self.execute_query(user, field_name, value, &self.create_upsert_query(table))
			.await
	}

	pub async fn increment<T>(
		&self,
		user: &Uuid,
		field_name: &str,
		value: T,
	) -> Result<(), sqlx::Error>
	where
		T: Add,
		T: TableName,
		T: for<'r> sqlx::Encode<'r, Postgres> + sqlx::Type<Postgres> + Send,
	{
		let table = T::table_name();
		self.execute_query(
			user,
			field_name,
			value,
			self.create_increment_query(table).as_str(),
		)
		.await
	}

	async fn execute_query<T>(
		&self,
		user: &Uuid,
		field_name: &str,
		value: T,
		query: &str,
	) -> Result<(), sqlx::Error>
	where
		T: for<'r> sqlx::Encode<'r, Postgres> + sqlx::Type<Postgres> + Send,
	{
		sqlx::query(query)
			.bind(user)
			.bind(field_name)
			.bind(value)
			.execute(&self.pg_pool)
			.await
			.map(|_| ())
	}

	fn create_upsert_query(&self, table: &str) -> String {
		format!(
			"insert into {table} ({user},{field_name},{field_value}) VALUES($1,$2, $3) ON CONFLICT ({user}, {field_name}) DO UPDATE SET {field_value}= $3",
			table = table,
			user = COLUMN_USER,
			field_name = COLUMN_FIELD_NAME,
			field_value = COLUMN_FIELD_VALUE,
		)
	}

	fn create_increment_query(&self, table: &str) -> String {
		format!(
			"update {table} set {field_value} = {field_value} + $3 where {user} = $1 and {field_name} = $2",
			table = table,
			user = COLUMN_USER,
			field_name = COLUMN_FIELD_NAME,
			field_value = COLUMN_FIELD_VALUE,
		)
	}
}
