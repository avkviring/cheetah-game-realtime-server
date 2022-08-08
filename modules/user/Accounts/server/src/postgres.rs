use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_postgres_pool(
	db: &str,
	user: &str,
	passwd: &str,
	host: &str,
	port: u16,
) -> PgPool {
	use std::time::Duration;
	let uri = format!("postgres://{}:{}@{}:{}/{}", user, passwd, host, port, db);
	let pool = PgPoolOptions::new()
		.max_connections(5)
		.acquire_timeout(Duration::from_secs(1))
		.connect(&uri)
		.await
		.unwrap();
	sqlx::migrate!().run(&pool).await.unwrap();
	pool
}

#[cfg(test)]
pub mod test {
	use std::collections::HashMap;

	use sqlx::PgPool;
	use testcontainers::clients::Cli;
	use testcontainers::images::postgres::Postgres;
	use testcontainers::Container;

	use crate::postgres::create_postgres_pool;

	lazy_static::lazy_static! {
		static ref CLI: Cli = Cli::docker();
	}
	pub async fn setup_postgresql() -> (PgPool, Container<'static, Postgres>) {
		let image = Postgres::default();
		let node = CLI.run(image);
		let port = node.get_host_port(5432);
		let pool = create_postgres_pool("postgres", "postgres", "", "127.0.0.1", port).await;
		(pool, node)
	}
}
