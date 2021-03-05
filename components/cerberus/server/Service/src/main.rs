use crate::server::run_grpc_server;

pub mod proto;
pub mod server;
pub mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    println!("±± cheetah game cerberus component ±±");

    let jwt_public_key = get_key_from_env("JWT_PUBLIC_KEY");
    let jwt_private_key = get_key_from_env("JWT_PRIVATE_KEY");
    let redis_host = get_env("CERBERUS_REDIS_HOST");
    let redis_port = get_env("CERBERUS_REDIS_PORT").parse().unwrap();

    let internal_service_port = get_env("CERBERUS_INTERNAL_GRPC_SERVICE_PORT")
        .parse()
        .unwrap();
    let external_service_port = get_env("CERBERUS_EXTERNAL_GRPC_SERVICE_PORT")
        .parse()
        .unwrap();

    run_grpc_server(
        jwt_public_key,
        jwt_private_key,
        redis_host,
        redis_port,
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
