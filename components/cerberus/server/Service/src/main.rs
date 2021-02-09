use futures::Future;
use tonic::transport::{Error, Server};

use crate::grpc::Cerberus;
use crate::server::run_grpc_server;

pub mod grpc;
pub mod proto;
pub mod server;
pub mod storage;
pub mod token;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jwt_public_key = get_key_from_env("JWT_PUBLIC_KEY");
    let jwt_private_key = get_key_from_env("JWT_PRIVATE_KEY");
    let redis_host = std::env::var("REDIS_HOST").expect("Env REDIS_HOST not set");
    let redis_port = std::env::var("REDIS_PORT")
        .expect("Env REDIS_PORT not set")
        .parse()
        .unwrap();

    run_grpc_server(jwt_public_key, jwt_private_key, redis_host, redis_port).await;
    Ok(())
}
fn get_key_from_env(name: &str) -> String {
    let value = std::env::var(name).expect(format!("Env {} not set", name).as_str());
    String::from_utf8(base64::decode(value).unwrap()).unwrap()
}
