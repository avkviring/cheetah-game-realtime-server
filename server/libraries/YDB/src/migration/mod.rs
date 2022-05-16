#[cfg(test)]
pub mod test {
	use ydb::Query;

	use crate::test_container::ydb_run_and_connect_for_test;

	#[tokio::test]
	pub async fn test() {
		// let (node, client) = ydb_run_and_connect_for_test().await;
		// client
		// 	.table_client()
		// 	.retry_transaction(|mut t| async move {
		// 		t.query(Query::new("scheme mkdir dir1")).await.unwrap();
		// 		Ok(())
		// 	})
		// 	.await
		// 	.unwrap();
	}
}
