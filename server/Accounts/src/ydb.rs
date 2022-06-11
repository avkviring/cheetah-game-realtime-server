use crate::users::user::User;

impl From<&User> for ydb::Value {
	fn from(user: &User) -> Self {
		ydb::Value::String(ydb::Bytes::from(user.0.as_bytes().to_vec()))
	}
}

#[cfg(test)]
pub mod test {
	use std::sync::Arc;

	use include_dir::include_dir;
	use uuid::Uuid;
	use ydb::Client;

	use cheetah_libraries_ydb::connect_to_ydb_and_prepare_schema;
	use cheetah_libraries_ydb::test_container::{get_or_create_ydb_instance, YDBTestInstance};

	pub async fn setup_ydb() -> (Client, Arc<YDBTestInstance>) {
		let db = Uuid::new_v4().to_string();
		let (instance, _) = get_or_create_ydb_instance(db.as_str()).await;

		let client = connect_to_ydb_and_prepare_schema(
			"/local",
			db.as_str(),
			"127.0.0.1",
			instance.get_port(),
			&include_dir!("$CARGO_MANIFEST_DIR/migrations"),
		)
		.await;
		(client, instance)
	}
}
