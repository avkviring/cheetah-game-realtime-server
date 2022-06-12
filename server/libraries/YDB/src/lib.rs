use crate::migration::Migrator;

pub mod builder;
pub mod converters;
pub mod error;
pub mod macros;
pub mod migration;
#[cfg(feature = "test_container")]
pub mod test_container;

#[cfg(test)]
pub mod tests {
	use include_dir::include_dir;

	use crate::migration::Migrator;
	use crate::test_container::get_or_create_ydb_instance;
	use crate::{query, select, update};

	#[tokio::test]
	async fn should_create_query() {
		let (_node, mut client) = get_or_create_ydb_instance("should_create_query").await;
		let mut migrator =
			Migrator::new_from_dir(&include_dir!("$CARGO_MANIFEST_DIR/test-migration"));
		migrator.migrate(&mut client).await.unwrap();

		let id = 124;
		update!(
			client.table_client(),
			query!("insert into a (id) values($id)", id=>id)
		)
		.await
		.unwrap();

		let result: Vec<i32> = select!(client.table_client(), query!("select * from a"), id=>i32)
			.await
			.unwrap();

		assert!(result.iter().any(|v| *v == id));
	}
}
