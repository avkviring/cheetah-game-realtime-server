use std::future::Future;

use tonic::transport::{Error, Server};

use crate::proto;
use crate::service::cerberus::Cerberus;

pub async fn run_grpc_server(
    jwt_public_key: String,
    jwt_private_key: String,
    redis_host: String,
    redis_port: u16,
    redis_auth: Option<String>,
    internal_port: u16,
    external_port: u16,
) {
    let internal = setup_internal(
        &jwt_public_key,
        &jwt_private_key,
        &redis_host,
        redis_port,
        redis_auth.clone(),
        internal_port,
    );
    let external = setup_external(
        &jwt_public_key,
        &jwt_private_key,
        &redis_host,
        redis_port,
        redis_auth,
        external_port,
    );
    tokio::try_join!(internal, external).unwrap();
}

fn setup_internal(
    jwt_public_key: &String,
    jwt_private_key: &String,
    redis_host: &String,
    redis_port: u16,
    redis_auth: Option<String>,
    port: u16,
) -> impl Future<Output = Result<(), Error>> {
    let internal_addr = format!("0.0.0.0:{}", port).parse().unwrap();

    let cerberus = Cerberus::new(
        jwt_private_key.clone(),
        jwt_public_key.clone(),
        redis_host.clone(),
        redis_port,
        redis_auth,
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
    redis_auth: Option<String>,
    port: u16,
) -> impl Future<Output = Result<(), Error>> {
    let external_addr = format!("0.0.0.0:{}", port).parse().unwrap();
    let external_service = proto::external::cerberus_server::CerberusServer::new(Cerberus::new(
        jwt_private_key.clone(),
        jwt_public_key.clone(),
        redis_host.clone(),
        redis_port,
        redis_auth,
    ));

    Server::builder()
        .add_service(external_service)
        .serve(external_addr)
}
