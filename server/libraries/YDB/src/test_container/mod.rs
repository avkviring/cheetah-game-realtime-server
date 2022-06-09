use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

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
		vec![
			WaitFor::Duration {
				length: Duration::from_secs(5),
			},
			WaitFor::Healthcheck,
		]
	}

	fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
		Box::new(self.env.iter())
	}
}

pub async fn get_or_create_ydb_instance(db: &str) -> (Arc<YDBTestInstance>, Client) {
	let ydb_instance = run_or_get_test_ydb_in_docker();
	let port = ydb_instance.get_port();
	let client = connect_to_ydb("local", port).await;
	let directory = format!("local/{}", db);
	client
		.scheme_client()
		.make_directory(directory.clone())
		.await
		.unwrap();
	(ydb_instance, connect_to_ydb(directory.as_str(), port).await)
}

async fn connect_to_ydb(db: &str, port: u16) -> Client {
	let url = format!("grpc://{}:{}", "127.0.0.1", port);
	let discovery = StaticDiscovery::from_str(url.as_str()).unwrap();
	let client = ClientBuilder::from_str(url)
		.unwrap()
		.with_database(db)
		.with_discovery(discovery)
		.client()
		.unwrap();
	client.wait().await.unwrap();
	client
}

lazy_static::lazy_static! {
	static ref DOCKER: clients::Cli= Default::default();
}
lazy_static::lazy_static! {
	static ref CONTAINER: Mutex<Weak<YDBTestInstance>> = Mutex::new(Weak::new());
}

pub struct YDBTestInstance {
	container: Container<'static, YDBImage>,
}
unsafe impl Send for YDBTestInstance {}
unsafe impl Sync for YDBTestInstance {}

impl YDBTestInstance {
	fn new() -> Self {
		Self {
			container: DOCKER.run(YDBImage::new()),
		}
	}
	pub fn get_port(&self) -> u16 {
		self.container.get_host_port(YDBImage::GRPC_PORT)
	}
}

///
/// Запустить если база не запущена и вернуть ссылку на контейнер
///
pub fn run_or_get_test_ydb_in_docker() -> Arc<YDBTestInstance> {
	let mut guard = CONTAINER.lock().unwrap();
	let container_holder = match guard.upgrade() {
		None => {
			let container_holder = Arc::new(YDBTestInstance::new());
			*guard = Arc::downgrade(&container_holder);
			container_holder
		}
		Some(container_holder) => container_holder,
	};
	container_holder
}

#[cfg(test)]
mod tests {
	use ydb::Query;

	use crate::test_container::get_or_create_ydb_instance;

	#[tokio::test]
	async fn should_create_docker_and_connect() {
		let (_node, client) = get_or_create_ydb_instance("should_create_docker_and_connect").await;
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
