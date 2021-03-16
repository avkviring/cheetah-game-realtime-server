use std::env;

use games_cheetah_auth_service::server::run_grpc_server;
use games_cheetah_auth_service::storage::pg::PgStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    println!("±± cheetah game auth component ±±");

    let pg_user = get_env("POSTGRES_USER");
    let pg_passwd = get_env("POSTGRES_PASSWORD");

    let pg_host = env::var("POSTGRES_HOST").unwrap_or("auth_postgres".to_owned());
    let pg_port = env::var("POSTGRES_PORT").unwrap_or("5432".to_owned());

    let service_port = env::var("INTERNAL_GRPC_SERVICE_PORT").unwrap_or("5000".to_owned());
    let cerberus_url = env::var("COMPONENT_CERBERUS_GRPC")
        .unwrap_or("http://component_cerberus_service:5000".to_owned());

    let google_client_id = get_env("AUTH_GOOGLE_CLIENT_ID");
    let jwt_public_key = get_env("JWT_PUBLIC_KEY");

    let storage = PgStorage::new(
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
    env::var(name).expect(format!("Env {}", name).as_str())
}
