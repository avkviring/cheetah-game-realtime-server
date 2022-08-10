use crate::storage::TableName;

impl TableName for i64 {
	fn table_name() -> &'static str {
		"cheetah_user_store_long_value"
	}
}

#[cfg(test)]
mod test {
	use crate::storage::test::{do_test_increment, do_test_set};

	#[tokio::test]
	async fn test_set() {
		do_test_set(666, 667).await
	}

	#[tokio::test]
	async fn test_increment() {
		do_test_increment(666).await
	}
}
