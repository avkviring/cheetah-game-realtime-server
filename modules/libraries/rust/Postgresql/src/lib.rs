use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_postgresql_pool_from_env() -> PgPool {
	let db_port = cheetah_libraries_microservice::get_env("POSTGRES_PORT");
	create_postgres_pool(
		cheetah_libraries_microservice::get_env("POSTGRES_DB").as_str(),
		cheetah_libraries_microservice::get_env("POSTGRES_USER").as_str(),
		cheetah_libraries_microservice::get_env("POSTGRES_PASSWORD").as_str(),
		cheetah_libraries_microservice::get_env("POSTGRES_HOST").as_str(),
		db_port
			.parse()
			.unwrap_or_else(|_| panic!("POSTGRES_PORT is wrong {:?}", db_port)),
	)
	.await
}

pub async fn create_postgres_pool(
	db: &str,
	user: &str,
	passwd: &str,
	host: &str,
	port: u16,
) -> PgPool {
	use std::time::Duration;
	let uri = format!("postgres://{}:{}@{}:{}/{}", user, passwd, host, port, db);
	println!("connect to {:?}", uri);
	PgPoolOptions::new()
		.max_connections(5)
		.acquire_timeout(Duration::from_secs(10))
		.connect(&uri)
		.await
		.unwrap()
}
