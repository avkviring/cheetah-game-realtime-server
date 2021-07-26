use crate::api::user;
use sqlx::{postgres::PgPoolOptions, types::ipnetwork::IpNetwork, PgPool};

pub async fn create_postgres_pool(
    db: &str,
    user: &str,
    passwd: &str,
    host: &str,
    port: u16,
) -> PgPool {
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

pub struct Storage {
    pool: PgPool,
}

impl From<PgPool> for Storage {
    fn from(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Storage {
    pub async fn attach(&self, user: user::Id, google_id: &str, ip: IpNetwork) {
        let mut tx = self.pool.begin().await.unwrap();

        sqlx::query("delete from google_users where user_id=$1 or google_id=$2")
            .bind(user)
            .bind(google_id)
            .execute(&mut tx)
            .await
            .unwrap();

        sqlx::query("insert into google_users values($1,$2, $3)")
            .bind(user)
            .bind(ip)
            .bind(google_id)
            .execute(&mut tx)
            .await
            .unwrap();

        sqlx::query("insert into google_users_history (ip, user_id, google_id) values($1,$2, $3)")
            .bind(ip)
            .bind(user)
            .bind(google_id)
            .execute(&mut tx)
            .await
            .unwrap();

        tx.commit().await.unwrap();
    }

    pub async fn find(&self, google_id: &str) -> Option<user::Id> {
        sqlx::query_as("select user_id from google_users where google_id=$1")
            .bind(google_id)
            .fetch_optional(&self.pool)
            .await
            .unwrap()
    }
}

#[cfg(test)]
pub mod tests {
    use super::Storage;
    use crate::api::user::Id as UserId;
    use cheetah_auth_user::storage::Storage as UserStorage;
    use chrono::NaiveDateTime;
    use sqlx::types::ipnetwork::IpNetwork;
    use sqlx::PgPool;
    use std::collections::HashMap;
    use std::str::FromStr;
    use testcontainers::images::postgres::Postgres;
    use testcontainers::{clients::Cli, Container, Docker as _};

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
        let storage = crate::storage::create_postgres_pool(
            "authentication",
            "authentication",
            "passwd",
            "127.0.0.1",
            port,
        )
        .await;

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
        storage.attach(user_a, "a@kviring.com", ip).await;
        storage.attach(user_b, "b@kviring.com", ip).await;

        assert_eq!(storage.find("a@kviring.com").await.unwrap(), user_a);
        assert_eq!(storage.find("b@kviring.com").await.unwrap(), user_b);
        assert!(storage.find("c@kviring.com").await.is_none());
    }

    #[tokio::test]
    pub async fn should_history() {
        let cli = Cli::default();
        let (pool, _node) = setup_postgresql_storage(&cli).await;
        let users = UserStorage::from(pool.clone());
        let storage = Storage::from(pool.clone());

        let ip = IpNetwork::from_str("127.0.0.1").unwrap();

        let user = users.create(ip).await.into();
        storage.attach(user, "a@kviring.com", ip).await;
        storage.attach(user, "b@kviring.com", ip).await;

        let result: Vec<(NaiveDateTime, UserId, String)> = sqlx::query_as(
            "select time, user_id, google_id from google_users_history order by time",
        )
        .fetch_all(&storage.pool)
        .await
        .unwrap();

        let i1 = result.get(0).unwrap();
        assert_eq!(i1.1, user);
        assert_eq!(i1.2, "a@kviring.com".to_owned());

        let i2 = result.get(1).unwrap();
        assert_eq!(i2.1, user);
        assert_eq!(i2.2, "b@kviring.com".to_owned());
    }

    /// Перепривязка google_id от одного пользователя к другому
    #[tokio::test]
    pub async fn should_reattach_1() {
        let cli = Cli::default();
        let (pool, _node) = setup_postgresql_storage(&cli).await;
        let users = UserStorage::from(pool.clone());
        let storage = Storage::from(pool.clone());

        let ip = IpNetwork::from_str("127.0.0.1").unwrap();

        let user_a = users.create(ip).await.into();
        let user_b = users.create(ip).await.into();
        let user_c = users.create(ip).await.into();

        storage.attach(user_a, "a@kviring.com", ip).await;
        storage.attach(user_b, "a@kviring.com", ip).await;
        storage.attach(user_c, "c@kviring.com", ip).await;

        assert_eq!(storage.find("a@kviring.com").await.unwrap(), user_b);
        // проверяем что данные других пользователей не изменились
        assert_eq!(storage.find("c@kviring.com").await.unwrap(), user_c);
    }

    /// Перепривязка google_id для пользователя
    #[tokio::test]
    pub async fn should_reattach_2() {
        let cli = Cli::default();
        let (pool, _node) = setup_postgresql_storage(&cli).await;
        let users = UserStorage::from(pool.clone());
        let storage = Storage::from(pool.clone());

        let ip = IpNetwork::from_str("127.0.0.1").unwrap();

        let user_a = users.create(ip).await.into();
        storage.attach(user_a, "a@kviring.com", ip).await;
        storage.attach(user_a, "aa@kviring.com", ip).await;

        let user_b = users.create(ip).await.into();
        storage.attach(user_b, "c@kviring.com", ip).await;

        assert!(storage.find("a@kviring.com").await.is_none());
        assert_eq!(storage.find("aa@kviring.com").await.unwrap(), user_a);

        // проверяем что данные другого пользователя не удалены
        assert_eq!(storage.find("c@kviring.com").await.unwrap(), user_b);
    }
}
