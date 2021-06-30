use crate::storage::cookie::mark_cookie_as_linked;
use crate::storage::pg::PgStorage;

pub async fn attach(
    storage: &PgStorage,
    player: u64,
    google_user_id: &str,
    ip: &ipnetwork::IpNetwork,
) {
    let mut tx = storage.pool.begin().await.unwrap();

    mark_cookie_as_linked(player, &mut tx).await;

    sqlx::query("delete from google_players where player=$1 or google_id=$2")
        .bind(player as i64)
        .bind(google_user_id)
        .execute(&mut tx)
        .await
        .unwrap();

    sqlx::query("insert into google_players values($1,$2, $3)")
        .bind(player as i64)
        .bind(ip)
        .bind(google_user_id)
        .execute(&mut tx)
        .await
        .unwrap();

    sqlx::query("insert into google_players_history (ip, player,google_id) values($1,$2, $3)")
        .bind(ip)
        .bind(player as i64)
        .bind(google_user_id)
        .execute(&mut tx)
        .await
        .unwrap();

    tx.commit().await.unwrap();
}

pub async fn find(storage: &PgStorage, google_id: &str) -> Option<u64> {
    let result: Result<Option<(i64,)>, sqlx::Error> =
        sqlx::query_as("select player from google_players where google_id=$1")
            .bind(google_id)
            .fetch_optional(&storage.pool)
            .await;
    result.map(|r| r.map(|v| v.0 as u64)).unwrap()
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use chrono::NaiveDateTime;
    use ipnetwork::IpNetwork;
    use testcontainers::clients::Cli;

    use crate::storage::cookie::FindResult;
    use crate::storage::google::{attach, find};
    use crate::storage::players::create_player;
    use crate::storage::test::setup_postgresql_storage;

    #[tokio::test]
    pub async fn should_attach() {
        let cli = Cli::default();
        let (storage, _node) = setup_postgresql_storage(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();
        let player_a = create_player(&storage, &ip).await;
        let player_b = create_player(&storage, &ip).await;
        attach(&storage, player_a, "a@kviring.com", &ip).await;
        attach(&storage, player_b, "b@kviring.com", &ip).await;

        assert_eq!(find(&storage, "a@kviring.com").await.unwrap(), player_a);
        assert_eq!(find(&storage, "b@kviring.com").await.unwrap(), player_b);
        assert!(find(&storage, "c@kviring.com").await.is_none());
    }

    #[tokio::test]
    pub async fn should_mark_cookie_as_linked_when_attach() {
        let cli = Cli::default();
        let (storage, _node) = setup_postgresql_storage(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();

        let player_a = create_player(&storage, &ip).await;
        let cookie_a = crate::storage::cookie::attach(&storage, player_a).await;

        let player_b = create_player(&storage, &ip).await;
        let cookie_b = crate::storage::cookie::attach(&storage, player_b).await;

        attach(&storage, player_a, "a@kviring.com", &ip).await;

        assert!(matches!(
            crate::storage::cookie::find(&storage, &cookie_a).await,
            FindResult::Linked
        ));
        assert!(matches!(
            crate::storage::cookie::find(&storage, &cookie_b).await,
            FindResult::Player(player) if player==player_b
        ));
    }

    #[tokio::test]
    pub async fn should_history() {
        let cli = Cli::default();
        let (storage, _node) = setup_postgresql_storage(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();
        let player = create_player(&storage, &ip).await;
        attach(&storage, player, "a@kviring.com", &ip).await;
        attach(&storage, player, "b@kviring.com", &ip).await;

        let result: Vec<(NaiveDateTime, i64, String)> = sqlx::query_as(
            "select time, player,google_id from google_players_history order by time",
        )
        .fetch_all(&storage.pool)
        .await
        .unwrap();

        let i1 = result.get(0).unwrap();
        assert_eq!(i1.1 as u64, player);
        assert_eq!(i1.2, "a@kviring.com".to_owned());

        let i2 = result.get(1).unwrap();
        assert_eq!(i2.1 as u64, player);
        assert_eq!(i2.2, "b@kviring.com".to_owned());
    }

    ///
    /// Перепривязка google_id от одного пользователя к другому
    ///
    #[tokio::test]
    pub async fn should_reattach_1() {
        let cli = Cli::default();
        let (storage, _node) = setup_postgresql_storage(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();
        let player_a = create_player(&storage, &ip).await;
        let player_b = create_player(&storage, &ip).await;
        let player_c = create_player(&storage, &ip).await;
        attach(&storage, player_a, "a@kviring.com", &ip).await;
        attach(&storage, player_b, "a@kviring.com", &ip).await;
        attach(&storage, player_c, "c@kviring.com", &ip).await;

        assert_eq!(find(&storage, "a@kviring.com").await.unwrap(), player_b);
        // проверяем что данные других пользователей не изменились
        assert_eq!(find(&storage, "c@kviring.com").await.unwrap(), player_c);
    }

    ///
    /// Перепривязка google_id для пользователя
    ///
    #[tokio::test]
    pub async fn should_reattach_2() {
        let cli = Cli::default();
        let (storage, _node) = setup_postgresql_storage(&cli).await;
        let ip = IpNetwork::from_str("127.0.0.1").unwrap();
        let player_a = create_player(&storage, &ip).await;
        attach(&storage, player_a, "a@kviring.com", &ip).await;
        attach(&storage, player_a, "aa@kviring.com", &ip).await;

        let player_b = create_player(&storage, &ip).await;
        attach(&storage, player_b, "c@kviring.com", &ip).await;

        assert!(find(&storage, "a@kviring.com").await.is_none());
        assert_eq!(find(&storage, "aa@kviring.com").await.unwrap(), player_a);

        // проверяем что данные другого пользователя не удалены
        assert_eq!(find(&storage, "c@kviring.com").await.unwrap(), player_b);
    }
}
