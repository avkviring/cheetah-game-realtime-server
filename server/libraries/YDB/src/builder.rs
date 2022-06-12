use include_dir::Dir;
use ydb::{Client, ClientBuilder, StaticDiscovery};

use cheetah_libraries_microservice::{get_env, get_env_or_default};

use crate::Migrator;

pub struct YdbClientBuilder {
	namespace: String,
	db_name: String,
	host: String,
	port: u16,
}

impl YdbClientBuilder {
	pub fn new(namespace: &str, db_name: &str, host: &str, port: u16) -> Self {
		Self {
			namespace: namespace.to_owned(),
			db_name: db_name.to_owned(),
			host: host.to_owned(),
			port,
		}
	}
	pub fn new_from_env(db_name: &str) -> Self {
		let namespace = get_env_or_default("YDB_NAMESPACE", "local");
		let host = get_env("YDB_HOST");
		let port: u16 = get_env("YDB_PORT")
			.parse()
			.unwrap_or_else(|_| panic!("Expect number but {}", get_env("YDB_PORT")));
		Self {
			namespace,
			db_name: db_name.to_owned(),
			host,
			port,
		}
	}

	pub async fn prepare_schema_and_build_client<'a>(self, migrations: &Dir<'a>) -> Client {
		let connection_url = format!("grpc://{}:{}", self.host, self.port);
		let database = format!("{}/{}", self.namespace, self.db_name);
		Self::create_db_if_not_exist(
			connection_url.as_str(),
			self.namespace.as_str(),
			database.as_str(),
		)
		.await;
		let mut client = Self::connect(connection_url.as_str(), database.as_str()).await;
		Migrator::new_from_dir(migrations)
			.migrate(&mut client)
			.await
			.unwrap();
		client
	}

	async fn connect(connection_url: &str, database: &str) -> Client {
		let discovery = StaticDiscovery::from_str(connection_url).unwrap();
		let client = ClientBuilder::from_str(connection_url)
			.unwrap()
			.with_database(database)
			.with_discovery(discovery)
			.client()
			.unwrap();
		client.wait().await.unwrap();
		client
	}

	async fn create_db_if_not_exist(connection_url: &str, namespace: &str, name: &str) {
		{
			let client = Self::connect(connection_url, namespace).await;
			client.wait().await.unwrap();
			client
				.scheme_client()
				.make_directory(name.to_owned())
				.await
				.unwrap();
		}
	}
}
