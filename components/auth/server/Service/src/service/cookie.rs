use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::{thread_rng, Rng};

use crate::service::storage::Storage;

pub async fn attach(storage: &Storage, player: i64, ip: &ipnetwork::IpNetwork) -> String {
    let mut tx = storage.pool.begin().await.unwrap();

    let cookie: String = OsRng
        .sample_iter(&Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();

    sqlx::query("insert into cookie_players (ip, player,cookie) values($1,$2,$3)")
        .bind(ip)
        .bind(player)
        .bind(&cookie)
        .execute(&mut tx)
        .await
        .unwrap();

    tx.commit().await.unwrap();
    cookie
}

pub async fn find(storage: &Storage, cookie: &str) -> Option<i64> {
    let result: Result<Option<(i64,)>, sqlx::Error> =
        sqlx::query_as("select player from cookie_players where cookie=$1")
            .bind(cookie)
            .fetch_optional(&storage.pool)
            .await;
    result.map(|r| r.map(|v| v.0)).unwrap()
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use ipnetwork::IpNetwork;
    use testcontainers::clients::Cli;
    use testcontainers::{images, Container, Docker};

    use crate::service::cookie::{attach, find};
    use crate::service::players::create_player;
    use crate::service::test::setup;

    #[tokio::test]
    pub async fn should_attach() {
        let cli = Cli::default();
        let (storage, _node) = setup(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();
        let player_a = create_player(&storage, &ip).await.unwrap();
        let player_b = create_player(&storage, &ip).await.unwrap();

        let cookie_a = attach(&storage, player_a, &ip).await;
        let cookie_b = attach(&storage, player_b, &ip).await;

        assert_eq!(find(&storage, cookie_a.as_str()).await.unwrap(), player_a);
        assert_eq!(find(&storage, cookie_b.as_str()).await.unwrap(), player_b);
    }
}
