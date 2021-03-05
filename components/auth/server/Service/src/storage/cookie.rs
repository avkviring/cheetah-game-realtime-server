use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::Rng;
use sqlx::Error;

use crate::storage::pg::PgStorage;

pub async fn attach(storage: &PgStorage, player: u64) -> String {
    loop {
        let cookie: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();
        match do_attach(storage, player, cookie.as_str()).await {
            Ok(_) => return cookie,
            Err(e) if AttachError::UniqueViolation == e => {}
            Err(e) => {
                panic!("{:?}", e)
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum AttachError {
    UniqueViolation,
    OtherError,
}

async fn do_attach(storage: &PgStorage, player: u64, cookie: &str) -> Result<(), AttachError> {
    let mut tx = storage.pool.begin().await.unwrap();
    let result: Result<_, sqlx::Error> =
        sqlx::query("insert into cookie_players (player,cookie) values($1,$2)")
            .bind(player as i64)
            .bind(&cookie)
            .execute(&mut tx)
            .await;

    let result = result
        .map_err(|e| {
            if let Error::Database(error) = e {
                if let Some(code) = error.code() {
                    if code == "23505" {
                        AttachError::UniqueViolation
                    } else {
                        AttachError::OtherError
                    }
                } else {
                    AttachError::OtherError
                }
            } else {
                AttachError::OtherError
            }
        })
        .map(|_| ());

    tx.commit().await.unwrap();
    result
}

pub async fn find(storage: &PgStorage, cookie: &str) -> Option<u64> {
    let result: Result<Option<(i64,)>, sqlx::Error> =
        sqlx::query_as("select player from cookie_players where cookie=$1")
            .bind(cookie)
            .fetch_optional(&storage.pool)
            .await;
    result.map(|r| r.map(|v| v.0 as u64)).unwrap()
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use ipnetwork::IpNetwork;
    use testcontainers::clients::Cli;

    use crate::storage::cookie::{attach, do_attach, find, AttachError};
    use crate::storage::players::create_player;
    use crate::storage::test::setup_postgresql_storage;

    #[tokio::test]
    pub async fn should_attach() {
        let cli = Cli::default();
        let (storage, _node) = setup_postgresql_storage(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();
        let player_a = create_player(&storage, &ip).await;
        let player_b = create_player(&storage, &ip).await;

        let cookie_a = attach(&storage, player_a).await;
        let cookie_b = attach(&storage, player_b).await;

        assert_eq!(find(&storage, cookie_a.as_str()).await.unwrap(), player_a);
        assert_eq!(find(&storage, cookie_b.as_str()).await.unwrap(), player_b);
    }

    #[tokio::test]
    pub async fn should_check_duplicate() {
        let cli = Cli::default();
        let (storage, _node) = setup_postgresql_storage(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();
        let player = create_player(&storage, &ip).await;
        assert!(do_attach(&storage, player, "cookie").await.is_ok());
        assert!(matches!(
            do_attach(&storage, player, "cookie").await,
            Result::Err(AttachError::UniqueViolation)
        ));
    }
}
