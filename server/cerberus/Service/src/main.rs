use crate::server::run_grpc_server;

pub mod proto;
pub mod server;
pub mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    log::info!("start cerberus");

    // ключи для генерации токенов
    let jwt_public_key = get_key_from_env("JWT_PUBLIC_KEY");
    let jwt_private_key = get_key_from_env("JWT_PRIVATE_KEY");

    // параметры redis
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("cerberus_redis".to_owned());
    let redis_auth = std::env::var("REDIS_AUTH").ok();
    let redis_port = std::env::var("REDIS_PORT")
        .unwrap_or("6379".to_owned())
        .parse()
        .unwrap();

    // порты grpc сервисов
    let internal_service_port = std::env::var("INTERNAL_GRPC_SERVICE_PORT")
        .unwrap_or("5001".to_owned())
        .parse()
        .unwrap();

    let external_service_port = std::env::var("EXTERNAL_GRPC_SERVICE_PORT")
        .unwrap_or("5000".to_owned())
        .parse()
        .unwrap();

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
fn get_key_from_env(name: &str) -> String {
    let value = get_env(name);
    String::from_utf8(base64::decode(value).unwrap()).unwrap()
}

fn get_env(name: &str) -> String {
    std::env::var(name).expect(format!("Env {}", name).as_str())
}
