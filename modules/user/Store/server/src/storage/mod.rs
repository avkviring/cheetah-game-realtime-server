use std::fmt::Display;

use include_dir::{include_dir, Dir};

pub use fetcher::Fetcher;
pub use updater::Updater;

mod double;
mod fetcher;
mod long;
mod string;
mod updater;

pub static COLUMN_USER: &str = "user_uuid";
pub static COLUMN_FIELD_NAME: &str = "field_name";
pub static COLUMN_FIELD_VALUE: &str = "value";

pub trait TableName {
	fn table_name() -> &'static str;
}

#[cfg(test)]
mod test {
	use std::fmt::Debug;
	use std::ops::Add;

	use sqlx::Postgres;
	use uuid::Uuid;

	use crate::postgres::test::setup_postgresql;
	use crate::storage::{Fetcher, TableName, Updater};

	pub async fn do_test_increment<T>(expected_value: T)
	where
		T: TableName,
		T: Copy,
		T: Add<Output = T>,
		T: PartialEq<T>,
		T: Debug,
		T: for<'r> sqlx::Encode<'r, Postgres> + sqlx::Type<Postgres> + Send,
		T: for<'r> sqlx::Decode<'r, Postgres> + sqlx::Type<Postgres>,
	{
		let (pg_pool, _instance) = setup_postgresql().await;
		let updater = Updater::new(pg_pool.clone());
		let fetcher = Fetcher::new(pg_pool.clone());

		let user_id = Uuid::new_v4();

		updater
			.set(&user_id, "points", expected_value)
			.await
			.unwrap();

		updater
			.increment(&user_id, "points", expected_value)
			.await
			.unwrap();

		let actual_value: T = fetcher.get(&user_id, "points").await.unwrap().unwrap();
		assert_eq!(expected_value + expected_value, actual_value)
	}

	pub async fn do_test_set<T>(expected_value_a: T, expected_value_b: T)
	where
		T: TableName,
		T: Clone,
		T: PartialEq<T>,
		T: Debug,
		T: for<'r> sqlx::Encode<'r, Postgres> + sqlx::Type<Postgres> + Send,
		T: for<'r> sqlx::Decode<'r, Postgres> + sqlx::Type<Postgres>,
	{
		let (pg_pool, _instance) = setup_postgresql().await;
		let updater = Updater::new(pg_pool.clone());
		let fetcher = Fetcher::new(pg_pool.clone());

		let user_id = Uuid::new_v4();

		updater
			.set(&user_id, "points", expected_value_a.clone())
			.await
			.unwrap();

		let actual_value: T = fetcher.get(&user_id, "points").await.unwrap().unwrap();
		assert_eq!(expected_value_a, actual_value);

		// проверяем двойную установку для тестирования upsert
		updater
			.set(&user_id, "points", expected_value_b.clone())
			.await
			.unwrap();

		let actual_value: T = fetcher.get(&user_id, "points").await.unwrap().unwrap();
		assert_eq!(expected_value_b, actual_value)
	}
}
