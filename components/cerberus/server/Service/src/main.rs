use futures::Future;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tonic::transport::{Error, Server};

use crate::grpc::Cerberus;

pub mod grpc;
pub mod proto;
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

    let internal = setup_internal(&jwt_public_key, &jwt_private_key, &redis_host, redis_port);
    let external = setup_external(&jwt_public_key, &jwt_private_key, &redis_host, redis_port);

    let (_, _) = futures::join!(internal, external);

    Ok(())
}

fn setup_internal(
    jwt_public_key: &String,
    jwt_private_key: &String,
    redis_host: &String,
    redis_port: u16,
) -> impl Future<Output = Result<(), Error>> {
    let internal_addr = "0.0.0.0:5001".parse().unwrap();

    let cerberus = Cerberus::new(
        jwt_private_key.clone(),
        jwt_public_key.clone(),
        redis_host.clone(),
        redis_port,
    );
    let internal_service = proto::internal::cerberus_server::CerberusServer::new(cerberus);
    Server::builder()
        .add_service(internal_service)
        .serve(internal_addr)
}

fn setup_external(
    jwt_public_key: &String,
    jwt_private_key: &String,
    redis_host: &String,
    redis_port: u16,
) -> impl Future<Output = Result<(), Error>> {
    let external_addr = "0.0.0.0:5002".parse().unwrap();
    let external_service = proto::external::cerberus_server::CerberusServer::new(Cerberus::new(
        jwt_private_key.clone(),
        jwt_public_key.clone(),
        redis_host.clone(),
        redis_port.clone(),
    ));

    Server::builder()
        .add_service(external_service)
        .serve(external_addr)
}

fn get_key_from_env(name: &str) -> String {
    String::from_utf8(
        base64::decode(std::env::var(name).expect(format!("Env {} not set", name).as_str()))
            .unwrap(),
    )
    .unwrap()
}
