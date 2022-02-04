use std::future::Future;

use tonic::transport::{Error, Server};

use crate::proto;
use crate::tokens::grpc::TokensGrpcService;

pub async fn run_grpc_server(
	jwt_public_key: String,
	jwt_private_key: String,
	redis_host: String,
	redis_port: u16,
	redis_auth: Option<String>,
) {
	let external = setup_external(&jwt_public_key, &jwt_private_key, &redis_host, redis_port, redis_auth);
	tokio::try_join!(external).unwrap();
}

fn setup_external(
	jwt_public_key: &str,
	jwt_private_key: &str,
	redis_host: &str,
	redis_port: u16,
	redis_auth: Option<String>,
) -> impl Future<Output = Result<(), Error>> {
	let external_addr = cheetah_microservice::get_external_service_binding_addr();
	let token_service = proto::cerberus::external::tokens_server::TokensServer::new(TokensGrpcService::new(
		jwt_private_key.to_string(),
		jwt_public_key.to_string(),
		redis_host.to_string(),
		redis_port,
		redis_auth,
	));

	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(token_service))
		.serve(external_addr)
}
