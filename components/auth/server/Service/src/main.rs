use std::time::Duration;

use sqlx::postgres::PgPoolOptions;

use games_cheetah_auth_service::server::run_grpc_server;
use games_cheetah_auth_service::storage::storage::Storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let pg_user = get_env("AUTH_POSTGRES_USER");
    let pg_passwd = get_env("AUTH_POSTGRES_PASSWORD");
    let pg_host = get_env("AUTH_POSTGRES_HOST");
    let pg_port = get_env("AUTH_POSTGRES_PORT");
    let service_port = get_env("AUTH_GRPC_SERVICE_PORT");
    let cerberus_url = get_env("AUTH_CERBERUS_URL");
    let google_client_id = get_env("AUTH_GOOGLE_CLIENT_ID");
    let jwt_public_key = get_env("JWT_PUBLIC_KEY");

    let storage = Storage::new(
        pg_user.as_str(),
        pg_passwd.as_str(),
        pg_host.as_str(),
        pg_port.parse().unwrap(),
    )
    .await;

    let parser = jsonwebtoken_google::Parser::new(google_client_id.as_str());

    run_grpc_server(
        storage,
        service_port.parse().unwrap(),
        cerberus_url.as_str(),
        parser,
        jwt_public_key,
    )
    .await;

    Ok(())
}

fn get_env(name: &str) -> String {
    std::env::var(name).expect(format!("Env {}", name).as_str())
}
