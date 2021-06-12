use crate::server::run_grpc_server;

pub mod proto;
pub mod server;
pub mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    println!("±± cheetah game cerberus component ±±");

    // ключи для генерации токенов
    let jwt_public_key = get_key_from_env("JWT_PUBLIC_KEY");
    let jwt_private_key = get_key_from_env("JWT_PRIVATE_KEY");

    // параметры redis
    let redis_host = get_env("CERBERUS_REDIS_MASTER_SERVICE_HOST");
    let redis_port = get_env("CERBERUS_REDIS_MASTER_SERVICE_PORT")
        .parse()
        .unwrap();
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
fn get_key_from_env(name: &str) -> String {
    let value = get_env(name);
    String::from_utf8(base64::decode(value).unwrap()).unwrap()
}

fn get_env(name: &str) -> String {
    std::env::var(name).expect(format!("Env {}", name).as_str())
}
