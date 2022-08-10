use crate::storage::TableName;

impl TableName for String {
	fn table_name() -> &'static str {
		"cheetah_user_store_string_value"
	}
}

#[cfg(test)]
mod test {
	use crate::storage::test::do_test_set;

	#[tokio::test]
	async fn test_set() {
		do_test_set("hello_a".to_string(), "hello_b".to_string()).await
	}
}
