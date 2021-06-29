use crate::server::run_grpc_server;

pub mod proto;
pub mod server;
pub mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    println!("±± cheetah game cerberus component ±±");

    // ключи для генерации токенов
    let jwt_public_key = get_env("JWT_PUBLIC_KEY");
    let jwt_private_key = get_env("JWT_PRIVATE_KEY");

    // параметры redis
    let redis_host = get_env("REDIS_HOST");
    let redis_port = get_env("REDIS_PORT").parse().unwrap();
    let redis_auth = std::env::var("REDIS_AUTH").ok();

    // порты grpc сервисов
    let external_service_port = 5000;
    let internal_service_port = 5001;

    run_grpc_server(
        jwt_public_key,
        jwt_private_key,
        redis_host,
        redis_port,
        redis_auth,
        internal_service_port,
        external_service_port,
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
