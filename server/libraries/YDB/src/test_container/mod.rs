use std::collections::HashMap;

use testcontainers::core::WaitFor;
use testcontainers::{clients, Container, Image};
use ydb::{Client, ClientBuilder, StaticDiscovery};

///
/// Yandex Data Base образ для интеграционных тестов
///  

#[derive(Default)]
pub struct YDBImage {
	env: HashMap<String, String>,
}

impl YDBImage {
	pub const GRPC_TLS_PORT: u16 = 2135;
	pub const GRPC_PORT: u16 = 2136;
	///
	/// WEB UI порт
	///
	pub const MON_PORT: u16 = 8765;

	pub fn new() -> Self {
		Self {
			env: vec![("YDB_USE_IN_MEMORY_PDISKS".to_owned(), "true".to_owned())]
				.into_iter()
				.collect(),
		}
	}
}

impl Image for YDBImage {
	type Args = ();

	fn name(&self) -> String {
		"cr.yandex/yc/yandex-docker-local-ydb".to_owned()
	}

	fn tag(&self) -> String {
		"latest".to_owned()
	}

	fn ready_conditions(&self) -> Vec<WaitFor> {
		vec![WaitFor::Healthcheck]
	}

	fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
		Box::new(self.env.iter())
	}
}

lazy_static::lazy_static! {
	static ref CLI: clients::Cli = Default::default();
}

pub async fn ydb_run_and_connect_for_test() -> (Container<'static, YDBImage>, Client) {
	let node = CLI.run(YDBImage::default());
	let port = node.get_host_port(YDBImage::GRPC_PORT);
	let url = format!("grpc://{}:{}?database=local", "127.0.0.1", port);
	let discovery = StaticDiscovery::from_str(url.as_str()).unwrap();
	let client = ClientBuilder::from_str(url)
		.unwrap()
		.with_discovery(discovery)
		.client()
		.unwrap();
	client.wait().await.unwrap();
	(node, client)
}

#[cfg(test)]
mod tests {
	use ydb::Query;

	use crate::test_container::ydb_run_and_connect_for_test;

	#[tokio::test]
	async fn should_create_docker_and_connect() {
		let (_node, client) = ydb_run_and_connect_for_test().await;
		let table_client = client.table_client();
		let value: i32 = table_client
			.retry_transaction(|mut t| async move {
				let value: i32 = t
					.query(Query::new("SELECT 2 + 3 as sum"))
					.await
					.unwrap()
					.into_only_row()
					.unwrap()
					.remove_field_by_name("sum")
					.unwrap()
					.try_into()
					.unwrap();
				Ok(value)
			})
			.await
			.unwrap();
		assert_eq!(value, 5);
	}
}
