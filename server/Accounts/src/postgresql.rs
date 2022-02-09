use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_postgres_pool(db: &str, user: &str, passwd: &str, host: &str, port: u16) -> PgPool {
	use std::time::Duration;
	let uri = format!("postgres://{}:{}@{}:{}/{}", user, passwd, host, port, db);
	PgPoolOptions::new()
		.max_connections(5)
		.connect_timeout(Duration::from_secs(10))
		.connect(&uri)
		.await
		.unwrap()
}

pub async fn migrate_db(pool: &PgPool) {
	sqlx::migrate!().run(pool).await.unwrap();
}

#[cfg(test)]
pub mod test {
	use std::collections::HashMap;

	use testcontainers::clients::Cli;
	use testcontainers::images::postgres::Postgres;
	use testcontainers::{Container, Docker};

	use crate::postgresql::{create_postgres_pool, migrate_db};
	use crate::PgPool;

	pub async fn setup_postgresql_storage(cli: &Cli) -> (PgPool, Container<'_, Cli, Postgres>) {
		let mut env = HashMap::default();
		env.insert("POSTGRES_USER".to_owned(), "authentication".to_owned());
		env.insert("POSTGRES_PASSWORD".to_owned(), "passwd".to_owned());
		let image = Postgres::default().with_version(13).with_env_vars(env);
		let node = cli.run(image);
		let port = node.get_host_port(5432).unwrap();
		let pool = create_postgres_pool("authentication", "authentication", "passwd", "127.0.0.1", port).await;
		migrate_db(&pool).await;
		(pool, node)
	}
}
