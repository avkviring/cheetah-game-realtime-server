use crate::grpc::Cerberus;
use crate::proto;
use std::future::Future;
use tonic::transport::{Error, Server};

pub async fn run_grpc_server(
    jwt_public_key: String,
    jwt_private_key: String,
    redis_host: String,
    redis_port: u16,
) {
    let internal = setup_internal(&jwt_public_key, &jwt_private_key, &redis_host, redis_port);
    let external = setup_external(&jwt_public_key, &jwt_private_key, &redis_host, redis_port);
    let (_, _) = tokio::join!(internal, external);
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
        redis_port,
    ));

    Server::builder()
        .add_service(external_service)
        .serve(external_addr)
}
