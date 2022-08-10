#[cfg(test)]
pub mod test {
	use sqlx::PgPool;
	use testcontainers::clients::Cli;
	use testcontainers::images::postgres::Postgres;
	use testcontainers::Container;

	use cheetah_libraries_postgresql::create_postgres_pool;

	lazy_static::lazy_static! {
		static ref CLI: Cli = Cli::docker();
	}
	pub async fn setup_postgresql() -> (PgPool, Container<'static, Postgres>) {
		let image = Postgres::default();
		let node = CLI.run(image);
		let port = node.get_host_port(5432);
		let pool = create_postgres_pool("postgres", "postgres", "", "127.0.0.1", port).await;
		sqlx::migrate!().run(&pool).await.unwrap();
		(pool, node)
	}
}
