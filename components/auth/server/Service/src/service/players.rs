use std::net::IpAddr;

use ipnetwork::IpNetwork;

use crate::service::storage::Storage;

///
/// Создать игрока, для каждого игры создается запись вида id, create_date
/// Все остальное храниться в отдельных таблицах
///
pub async fn create_player(storage: &Storage, ip: &IpNetwork) -> Result<i64, sqlx::Error> {
    let result: Result<(i64,), sqlx::Error> =
        sqlx::query_as("insert into players (ip) values ($1) returning id")
            .bind(ip)
            .fetch_one(&storage.pool)
            .await;

    result.map(|it| it.0)
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use chrono::NaiveDateTime;
    use ipnetwork::IpNetwork;
    use testcontainers::clients::Cli;
    use testcontainers::{images, Container, Docker};

    use crate::service::players::create_player;
    use crate::service::test::setup;

    #[tokio::test]
    pub async fn test() {
        let cli = Cli::default();
        let (storage, _node) = setup(&cli).await;
        let addr_a = IpNetwork::from_str("127.1.0.1").unwrap();
        let id1 = create_player(&storage, &addr_a).await.unwrap();
        let addr_b = IpNetwork::from_str("127.0.0.1").unwrap();
        let id2 = create_player(&storage, &addr_b).await.unwrap();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let result: Vec<(i64, IpNetwork, NaiveDateTime)> =
            sqlx::query_as("select id, ip, time from players")
                .fetch_all(&storage.pool)
                .await
                .unwrap();

        assert_eq!(result[0].0, 1);
        assert_eq!(result[0].1, addr_a);

        assert_eq!(result[1].0, 2);
        assert_eq!(result[1].1, addr_b);
    }
}
