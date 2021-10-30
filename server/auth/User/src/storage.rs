use sqlx::{postgres::PgPoolOptions, types::ipnetwork::IpNetwork, PgPool};

pub async fn create_postgres_pool(db: &str, user: &str, passwd: &str, host: &str, port: u16) -> PgPool {
	use std::time::Duration;
	let uri = format!("postgres://{}:{}@{}:{}/{}", user, passwd, host, port, db);
	let pool = PgPoolOptions::new()
		.max_connections(5)
		.connect_timeout(Duration::from_secs(10))
		.connect(&uri)
		.await
		.unwrap();

	migrate_db(&pool).await;

	pool
}

pub async fn migrate_db(pool: &PgPool) {
	sqlx::migrate!().run(pool).await.unwrap();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::FromRow, sqlx::Type)]
#[sqlx(transparent)]
pub struct Id(i64);

impl From<Id> for i64 {
	fn from(id: Id) -> Self {
		id.0
	}
}

impl From<i64> for Id {
	fn from(id: i64) -> Self {
		Self(id)
	}
}

#[derive(Clone)]
pub struct Storage {
	pool: PgPool,
}

impl From<PgPool> for Storage {
	fn from(pool: PgPool) -> Self {
		Self { pool }
	}
}

impl Storage {
	/// Создать игрока, для каждого игры создается запись вида `id, create_date`
	/// Все остальное храниться в отдельных таблицах
	pub async fn create(&self, ip: IpNetwork) -> Id {
		sqlx::query_as("insert into users (ip) values ($1) returning id")
			.bind(ip)
			.fetch_one(&self.pool)
			.await
			.unwrap()
	}
}

#[cfg(test)]
pub mod tests {
	use super::{Id, Storage};
	use chrono::NaiveDateTime;
	use sqlx::types::ipnetwork::IpNetwork;
	use sqlx::PgPool;
	use std::collections::HashMap;
	use testcontainers::images::postgres::Postgres;
	use testcontainers::{clients::Cli, Container, Docker as _};

	pub async fn setup_postgresql_storage(cli: &Cli) -> (PgPool, Container<'_, Cli, Postgres>) {
		let mut env = HashMap::default();
		env.insert("POSTGRES_USER".to_owned(), "authentication".to_owned());
		env.insert("POSTGRES_PASSWORD".to_owned(), "passwd".to_owned());
		let image = Postgres::default().with_version(13).with_env_vars(env);
		let node = cli.run(image);
		let port = node.get_host_port(5432).unwrap();
		let storage = crate::storage::create_postgres_pool("authentication", "authentication", "passwd", "127.0.0.1", port).await;
		(storage, node)
	}

	#[tokio::test]
	pub async fn should_create() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let users = Storage::from(pool.clone());

		let addr_a = "127.1.0.1".parse().unwrap();
		let id_a = users.create(addr_a).await;
		assert_eq!(id_a, Id(1));

		let addr_b = "127.1.0.1".parse().unwrap();
		let id_b = users.create(addr_b).await;
		assert_eq!(id_b, Id(2));

		let result: Vec<(Id, IpNetwork, NaiveDateTime)> = sqlx::query_as("select id, ip, create_time from users")
			.fetch_all(&pool)
			.await
			.unwrap();

		assert_eq!(result[0].0, id_a);
		assert_eq!(result[0].1, addr_a);

		assert_eq!(result[1].0, id_b);
		assert_eq!(result[1].1, addr_b);
	}
}
