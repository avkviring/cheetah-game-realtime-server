use include_dir::include_dir;
use ydb::{Client, ClientBuilder, StaticDiscovery};

use cheetah_libraries_ydb::migration::Migrator;

use crate::users::user::User;

pub async fn connect_to_ydb_and_prepare_schema(db: &str, host: &str, port: u16) -> Client {
	let url = format!("grpc://{}:{}", host, port);
	let db = format!("/local/{}", db);
	create_db(&url, &db).await;
	let mut client = create_client(url, db).await;
	Migrator::new_from_dir(include_dir!("$CARGO_MANIFEST_DIR/migrations/"))
		.migrate(&mut client)
		.await
		.unwrap();
	client
}

async fn create_client(url: String, db: String) -> Client {
	let discovery = StaticDiscovery::from_str(url.as_str()).unwrap();
	let mut client = ClientBuilder::from_str(url)
		.unwrap()
		.with_database(db)
		.with_discovery(discovery)
		.client()
		.unwrap();
	client.wait().await.unwrap();
	client
}

async fn create_db(url: &String, db: &String) {
	{
		let discovery = StaticDiscovery::from_str(url.as_str()).unwrap();
		let mut client = ClientBuilder::from_str(url.clone())
			.unwrap()
			.with_database("/local/")
			.with_discovery(discovery)
			.client()
			.unwrap();
		client.wait().await.unwrap();
		client
			.scheme_client()
			.make_directory(db.clone())
			.await
			.unwrap();
	}
}

impl From<&User> for ydb::Value {
	fn from(user: &User) -> Self {
		ydb::Value::String(ydb::Bytes::from(user.0.as_bytes().to_vec()))
	}
}

#[cfg(test)]
pub mod test {
	use std::sync::Arc;

	use uuid::Uuid;
	use ydb::Client;

	use cheetah_libraries_ydb::test_container::{get_or_create_ydb_instance, YDBTestInstance};

	use crate::ydb::connect_to_ydb_and_prepare_schema;

	pub async fn setup_ydb() -> (Client, Arc<YDBTestInstance>) {
		let db = Uuid::new_v4().to_string();
		let (instance, _) = get_or_create_ydb_instance(db.as_str()).await;
		let client =
			connect_to_ydb_and_prepare_schema(db.as_str(), "127.0.0.1", instance.get_port()).await;
		(client, instance)
	}
}
