use std::env;

use games_cheetah_authentication_service::server::run_grpc_server;
use games_cheetah_authentication_service::storage::pg::PgStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    println!("±± cheetah game authentication component ±±");

    let pg_user = get_env("POSTGRES_USER");
    let pg_passwd = get_env("POSTGRES_PASSWORD");

    let pg_host = "authentication_postgres".to_owned();
    let pg_port = "5432".to_owned();

    let cerberus_url = "http://cerberus:5001".to_owned();

    let google_token_parser = env::var("AUTH_GOOGLE_CLIENT_ID")
        .ok()
        .map(|google_client_id| jsonwebtoken_google::Parser::new(google_client_id.as_str()));
    let jwt_public_key = get_env("JWT_PUBLIC_KEY");

    let storage = PgStorage::new(
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
    env::var(name).expect(format!("Env {}", name).as_str())
}
