use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::api::user;

pub async fn create_postgres_pool(db: &str, user: &str, passwd: &str, host: &str, port: u16) -> PgPool {
	use std::time::Duration;
	let uri = format!("postgres://{}:{}@{}:{}/{}", user, passwd, host, port, db);
	PgPoolOptions::new()
		.max_connections(5)
		.connect_timeout(Duration::from_secs(5))
		.connect(&uri)
		.await
		.unwrap()
}

pub async fn migrate_db(pool: &PgPool) {
	sqlx::migrate!().run(pool).await.unwrap();
}

#[derive(Debug)]
enum AttachError {
	UniqueViolation,
	Query(sqlx::Error),
}

pub enum FindResult {
	NotFound,
	Player(user::Id),
	Linked,
}

pub struct Storage {
	pool: PgPool,
}

impl From<PgPool> for Storage {
	fn from(pool: PgPool) -> Self {
		Self { pool }
	}
}

impl Storage {
	pub async fn attach(&self, user: user::Id) -> String {
		use rand::distributions::Alphanumeric;
		use rand::rngs::OsRng;
		use rand::Rng;

		loop {
			let cookie: String = OsRng.sample_iter(&Alphanumeric).take(128).map(char::from).collect();

			match self.do_attach(user, &cookie).await {
				Ok(_) => break cookie,
				Err(AttachError::UniqueViolation) => continue,
				Err(err) => panic!("{:?}", err),
			}
		}
	}

	async fn do_attach(&self, user: user::Id, cookie: &str) -> Result<(), AttachError> {
		let mut tx = self.pool.begin().await.unwrap();

		let result: Result<_, sqlx::Error> = sqlx::query("insert into cookie_users (user_id,cookie) values($1,$2)")
			.bind(user as user::Id)
			.bind(cookie)
			.execute(&mut tx)
			.await;

		tx.commit().await.unwrap();

		result.map(|_| ()).map_err(|err| match err {
			sqlx::Error::Database(err) if matches!(err.code().as_deref(), Some("23505")) => AttachError::UniqueViolation,
			err => AttachError::Query(err),
		})
	}

	pub async fn find(&self, cookie: &str) -> FindResult {
		sqlx::query_as("select user_id, linked from cookie_users where cookie=$1")
			.bind(cookie)
			.fetch_optional(&self.pool)
			.await
			.unwrap()
			.map_or(
				FindResult::NotFound,
				|(user, linked)| {
					if linked {
						FindResult::Linked
					} else {
						FindResult::Player(user)
					}
				},
			)
	}

	pub async fn link_cookie(&self, user: user::Id) {
		sqlx::query("update cookie_users set linked=true where user_id=$1")
			.bind(user)
			.fetch_optional(&self.pool)
			.await
			.unwrap();
	}

	pub async fn _mark_cookie_as_linked(user: user::Id, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) {
		sqlx::query("update cookie_users set linked=true where user_id=$1")
			.bind(user)
			.execute(tx)
			.await
			.unwrap();
	}
}

#[cfg(test)]
pub mod tests {
	use std::collections::HashMap;
	use std::str::FromStr;

	use sqlx::types::ipnetwork::IpNetwork;
	use sqlx::PgPool;
	use testcontainers::images::postgres::Postgres;
	use testcontainers::{clients::Cli, Container, Docker as _};

	use cheetah_auth_user::storage::Storage as UserStorage;

	use crate::api::user::Id as UserId;

	use super::{AttachError, FindResult, Storage};

	impl From<cheetah_auth_user::storage::Id> for UserId {
		fn from(id: cheetah_auth_user::storage::Id) -> Self {
			let id: i64 = id.into();
			Self::from(id as u64)
		}
	}

	pub async fn setup_postgresql_storage(cli: &Cli) -> (PgPool, Container<'_, Cli, Postgres>) {
		let mut env = HashMap::default();
		env.insert("POSTGRES_USER".to_owned(), "authentication".to_owned());
		env.insert("POSTGRES_PASSWORD".to_owned(), "passwd".to_owned());
		let image = Postgres::default().with_version(13).with_env_vars(env);
		let node = cli.run(image);
		let port = node.get_host_port(5432).unwrap();
		let storage = crate::storage::create_postgres_pool("authentication", "authentication", "passwd", "127.0.0.1", port).await;

		// FIXME(lain-dono): version mismatch error
		// нельзя выполнить две миграции
		// cheetah_auth_user::storage::migrate_db(&storage).await;

		sqlx::query(
			r"
create table if not exists users (
    id          bigserial not null constraint users_pk primary key,
    ip          inet not null,
    create_time timestamp default CURRENT_TIMESTAMP not null
)",
		)
		.execute(&storage)
		.await
		.unwrap();

		sqlx::query(r"create index users_id_uindex on users (id)")
			.execute(&storage)
			.await
			.unwrap();

		super::migrate_db(&storage).await;

		(storage, node)
	}

	#[tokio::test]
	pub async fn should_attach() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let users = UserStorage::from(pool.clone());
		let storage = Storage::from(pool.clone());

		let ip = IpNetwork::from_str("127.0.0.1").unwrap();

		let user_a = users.create(ip).await.into();
		let user_b = users.create(ip).await.into();

		let cookie_a = storage.attach(user_a).await;
		let cookie_b = storage.attach(user_b).await;

		assert!(matches!(storage.find( &cookie_a).await,
                FindResult::Player(user) if user == user_a));
		assert!(matches!(storage.find( &cookie_b).await,
            FindResult::Player(user) if user == user_b));
	}

	#[tokio::test]
	pub async fn should_linked() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let users = UserStorage::from(pool.clone());
		let storage = Storage::from(pool.clone());

		let ip = IpNetwork::from_str("127.0.0.1").unwrap();

		let user = users.create(ip).await.into();
		let cookie = storage.attach(user).await;

		storage.link_cookie(user).await;

		assert!(matches!(storage.find(&cookie).await, FindResult::Linked));
	}

	#[tokio::test]
	pub async fn should_check_duplicate() {
		let cli = Cli::default();
		let (pool, _node) = setup_postgresql_storage(&cli).await;
		let users = UserStorage::from(pool.clone());
		let storage = Storage::from(pool.clone());

		let ip = IpNetwork::from_str("127.0.0.1").unwrap();
		let user = users.create(ip).await.into();
		assert!(storage.do_attach(user, "cookie").await.is_ok());
		assert!(matches!(storage.do_attach(user, "cookie").await, Err(AttachError::UniqueViolation)));
	}
}
