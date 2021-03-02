use std::collections::HashMap;
use std::time::Duration;

use testcontainers::clients::Cli;
use testcontainers::images::postgres::Postgres;
use testcontainers::images::redis::Redis;
use testcontainers::{images, Container, Docker};
use tokio::task::JoinHandle;

use games_cheetah_auth_service::storage::storage::Storage;
use games_cheetah_cerberus_service::test_helper;

async fn setup_postgresql_storage<'a>(cli: &'a Cli) -> (Storage, Container<'a, Cli, Postgres>) {
    let mut env = HashMap::default();
    env.insert("POSTGRES_USER".to_owned(), "auth".to_owned());
    env.insert("POSTGRES_PASSWORD".to_owned(), "passwd".to_owned());
    let image = images::postgres::Postgres::default()
        .with_version(12)
        .with_env_vars(env);
    let node = cli.run(image);
    let port = node.get_host_port(5432).unwrap();
    let storage = Storage::new("auth", "passwd", "127.0.0.1", port).await;
    (storage, node)
}

pub async fn setup<'a>(
    cli: &'a Cli,
    internal_cerberus_port: u16,
    external_cerberus_port: u16,
    service_port: u16,
    public_google_key_url: String,
) -> (
    Container<'a, Cli, Redis>,
    Container<'a, Cli, Postgres>,
    JoinHandle<()>,
    JoinHandle<()>,
) {
    let (handler_cerberus, redis_container) =
        test_helper::stub_grpc_server(internal_cerberus_port, external_cerberus_port).await;

    let (storage, postgres_container) = setup_postgresql_storage(cli).await;
    let handler_auth = tokio::spawn(async move {
        games_cheetah_auth_service::server::run_grpc_server(
            storage,
            service_port,
            format!("http://127.0.0.1:{}", internal_cerberus_port).as_str(),
            jsonwebtoken_google::Parser::new_with_custom_cert_url(
                jsonwebtoken_google::test_helper::CLIENT_ID,
                public_google_key_url.as_str(),
            ),
        )
        .await;
    });

    tokio::time::sleep(Duration::from_secs(1)).await;
    (
        redis_container,
        postgres_container,
        handler_cerberus,
        handler_auth,
    )
}
