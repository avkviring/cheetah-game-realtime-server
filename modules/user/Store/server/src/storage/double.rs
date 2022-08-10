use uuid::Uuid;

use crate::storage::{Fetcher, TableName};

impl TableName for f64 {
	fn table_name() -> &'static str {
		"cheetah_user_store_double_value"
	}
}

#[cfg(test)]
mod test {
	use crate::storage::test::{do_test_increment, do_test_set};

	#[tokio::test]
	async fn test_set() {
		do_test_set(666.666, 667.667).await
	}

	#[tokio::test]
	async fn test_increment() {
		do_test_increment(666.666).await
	}
}
