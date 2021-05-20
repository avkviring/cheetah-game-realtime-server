use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct PgStorage {
    pub pool: Pool<Postgres>,
}

impl PgStorage {
    pub async fn new(pg_user: &str, pg_passwd: &str, pg_host: &str, pg_port: u16) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_timeout(Duration::from_secs(5))
            .connect(
                format!(
                    "postgres://{}:{}@{}:{}/authentication",
                    pg_user, pg_passwd, pg_host, pg_port
                )
                .as_str(),
            )
            .await
            .unwrap();

        sqlx::migrate!().run(&pool).await.unwrap();
        Self { pool }
    }
}
