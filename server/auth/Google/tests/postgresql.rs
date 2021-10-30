use std::collections::HashMap;
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use testcontainers::clients::Cli;
use testcontainers::images::postgres::Postgres;
use testcontainers::{images, Container, Docker};

pub async fn create_psql_databases(cli: &Cli, count: usize) -> (Vec<sqlx::PgPool>, Container<'_, Cli, Postgres>) {
	let mut env = HashMap::default();
	env.insert("POSTGRES_USER".to_owned(), "creator".to_owned());
	env.insert("POSTGRES_PASSWORD".to_owned(), "passwd".to_owned());
	let image = images::postgres::Postgres::default().with_version(13).with_env_vars(env);
	let container = cli.run(image);
	let port = container.get_host_port(5432).unwrap();

	let uri = format!("postgres://creator:passwd@127.0.0.1:{}/creator", port);
	let pool = PgPoolOptions::new()
		.max_connections(5)
		.connect_timeout(Duration::from_secs(5))
		.connect(&uri)
		.await
		.unwrap();

	let mut result = Vec::new();
	for i in 0..count {
		sqlx::query(format!("CREATE DATABASE db_{} with owner=creator", i).as_str())
			.execute(&pool)
			.await
			.unwrap();
		let uri = format!("postgres://creator:passwd@127.0.0.1:{}/db_{}", port, i);
		let pool = PgPoolOptions::new()
			.max_connections(5)
			.connect_timeout(Duration::from_secs(5))
			.connect(&uri)
			.await
			.unwrap();
		result.push(pool)
	}

	(result, container)
}
