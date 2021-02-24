use prost::bytes::Buf;

use crate::service::storage::Storage;

pub async fn attach(storage: &Storage, player: i64, email: &str, ip: &ipnetwork::IpNetwork) {
    let mut tx = storage.pool.begin().await.unwrap();

    sqlx::query("delete from cookie_players where player=$1")
        .bind(player)
        .execute(&mut tx)
        .await
        .unwrap();

    sqlx::query("delete from google_players where player=$1 or email=$2")
        .bind(player)
        .bind(email)
        .execute(&mut tx)
        .await
        .unwrap();

    sqlx::query("insert into google_players values($1,$2, $3)")
        .bind(player)
        .bind(ip)
        .bind(email)
        .execute(&mut tx)
        .await
        .unwrap();

    sqlx::query("insert into google_players_history (ip, player,email) values($1,$2, $3)")
        .bind(ip)
        .bind(player)
        .bind(email)
        .execute(&mut tx)
        .await
        .unwrap();

    tx.commit().await.unwrap();
}

pub async fn find(storage: &Storage, email: &str) -> Option<i64> {
    let result: Result<Option<(i64,)>, sqlx::Error> =
        sqlx::query_as("select player from google_players where email=$1")
            .bind(email)
            .fetch_optional(&storage.pool)
            .await;
    result.map(|r| r.map(|v| v.0)).unwrap()
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use chrono::NaiveDateTime;
    use ipnetwork::IpNetwork;
    use testcontainers::clients::Cli;
    use testcontainers::{images, Container, Docker};

    use crate::service::google::{attach, find};
    use crate::service::players::create_player;
    use crate::service::test::setup;

    #[tokio::test]
    pub async fn should_attach() {
        let cli = Cli::default();
        let (storage, _node) = setup(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();
        let player_a = create_player(&storage, &ip).await.unwrap();
        let player_b = create_player(&storage, &ip).await.unwrap();
        attach(&storage, player_a, "a@kviring.com", &ip).await;
        attach(&storage, player_b, "b@kviring.com", &ip).await;

        assert_eq!(find(&storage, "a@kviring.com").await.unwrap(), player_a);
        assert_eq!(find(&storage, "b@kviring.com").await.unwrap(), player_b);
        assert!(find(&storage, "c@kviring.com").await.is_none());
    }

    #[tokio::test]
    pub async fn should_delete_cookie_when_attach() {
        let cli = Cli::default();
        let (storage, _node) = setup(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();

        let player_a = create_player(&storage, &ip).await.unwrap();
        let cookie_a = crate::service::cookie::attach(&storage, player_a, &ip).await;

        let player_b = create_player(&storage, &ip).await.unwrap();
        let cookie_b = crate::service::cookie::attach(&storage, player_b, &ip).await;

        attach(&storage, player_a, "a@kviring.com", &ip).await;

        assert!(crate::service::cookie::find(&storage, &cookie_a)
            .await
            .is_none());
        assert_eq!(
            crate::service::cookie::find(&storage, &cookie_b)
                .await
                .unwrap(),
            player_b
        );
    }

    #[tokio::test]
    pub async fn should_history() {
        let cli = Cli::default();
        let (storage, _node) = setup(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();
        let player = create_player(&storage, &ip).await.unwrap();
        attach(&storage, player, "a@kviring.com", &ip).await;
        attach(&storage, player, "b@kviring.com", &ip).await;

        let result: Vec<(NaiveDateTime, i64, String)> =
            sqlx::query_as("select time, player,email from google_players_history order by time")
                .fetch_all(&storage.pool)
                .await
                .unwrap();

        let i1 = result.get(0).unwrap();
        assert_eq!(i1.1, player);
        assert_eq!(i1.2, "a@kviring.com".to_owned());

        let i2 = result.get(1).unwrap();
        assert_eq!(i2.1, player);
        assert_eq!(i2.2, "b@kviring.com".to_owned());
    }

    ///
    /// Перепривязка email от одного пользователя к другому
    ///
    #[tokio::test]
    pub async fn should_reattach_1() {
        let cli = Cli::default();
        let (storage, _node) = setup(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();
        let player_a = create_player(&storage, &ip).await.unwrap();
        let player_b = create_player(&storage, &ip).await.unwrap();
        let player_c = create_player(&storage, &ip).await.unwrap();
        attach(&storage, player_a, "a@kviring.com", &ip).await;
        attach(&storage, player_b, "a@kviring.com", &ip).await;
        attach(&storage, player_c, "c@kviring.com", &ip).await;

        assert_eq!(find(&storage, "a@kviring.com").await.unwrap(), player_b);
        // проверяем что данные других пользователей не изменились
        assert_eq!(find(&storage, "c@kviring.com").await.unwrap(), player_c);
    }

    ///
    /// Перепривязка email для пользователя
    ///
    #[tokio::test]
    pub async fn should_reattach_2() {
        let cli = Cli::default();
        let (storage, _node) = setup(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();
        let player_a = create_player(&storage, &ip).await.unwrap();
        attach(&storage, player_a, "a@kviring.com", &ip).await;
        attach(&storage, player_a, "aa@kviring.com", &ip).await;

        let player_b = create_player(&storage, &ip).await.unwrap();
        attach(&storage, player_b, "c@kviring.com", &ip).await;

        assert!(find(&storage, "a@kviring.com").await.is_none());
        assert_eq!(find(&storage, "aa@kviring.com").await.unwrap(), player_a);

        // проверяем что данные другого пользователя не удалены
        assert_eq!(find(&storage, "c@kviring.com").await.unwrap(), player_b);
    }
}
