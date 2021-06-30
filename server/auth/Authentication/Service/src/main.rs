use std::env;

use cheetah_auth_authentication::server::run_grpc_server;
use cheetah_auth_authentication::storage::pg::PgStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    println!("±± cheetah game authentication component ±±");

    let pg_user = get_env("POSTGRES_USER");
    let pg_passwd = get_env("POSTGRES_PASSWORD");
    let pg_db = get_env("POSTGRES_DB");
    let pg_host = get_env("POSTGRES_HOST");
    let pg_port = get_env("POSTGRES_PORT");
    let cerberus_host = get_env("CERBERUS_INTERNAL_HOST");
    let cerberus_port = get_env("CERBERUS_INTERNAL_PORT");

    let cerberus_url = format!("http://{}:{}", cerberus_host, cerberus_port);

    let google_token_parser = env::var("AUTH_GOOGLE_CLIENT_ID")
        .ok()
        .map(|google_client_id| jsonwebtoken_google::Parser::new(google_client_id.as_str()));
    let jwt_public_key = get_env("JWT_PUBLIC_KEY");

    let storage = PgStorage::new(
        pg_db.as_str(),
        pg_user.as_str(),
        pg_passwd.as_str(),
        pg_host.as_str(),
        pg_port.parse().unwrap(),
    )
    .await;

    run_grpc_server(
        storage,
        jwt_public_key,
        cerberus_url.as_str(),
        5000,
        env::var("COOKIE").is_ok(),
        google_token_parser,
    )
    .await;

    Ok(())
}

fn get_env(name: &str) -> String {
    let value = std::env::var(name).unwrap_or_else(|_| panic!("Env {}", name));
    if value.trim().is_empty() {
        panic!("Env {} is empty", name);
    }
    value
}
