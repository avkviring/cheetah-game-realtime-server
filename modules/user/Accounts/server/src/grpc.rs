use sqlx::PgPool;
use tonic::transport::Server;
use tonic_health::ServingStatus;

use crate::cookie::service::CookieService;
use crate::proto::cookie_server::CookieServer;
use crate::proto::tokens_server::TokensServer;
use crate::tokens::grpc::TokensGrpcService;
use crate::tokens::TokensService;
use crate::users::service::UserService;

pub async fn run_grpc_server(jwt_public_key: String, jwt_private_key: String, pg_pool: PgPool) {
	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

	health_reporter.set_service_status("", ServingStatus::Serving).await;
	let token_service = TokensService::new(pg_pool.clone(), jwt_private_key, jwt_public_key.clone()).await;
	let user_service = UserService::new(pg_pool.clone());

	let token_grpc_service = TokensServer::new(TokensGrpcService::new(token_service.clone()));
	let cookie_grpc_service = CookieServer::new(CookieService::new(pg_pool.clone(), token_service.clone(), user_service.clone()));

	// если мы здесь - то соединение к базе установлены, все параметры заданы
	// то есть мы можем сказать что сервисы тоже готовы
	health_reporter.set_serving::<TokensServer<TokensGrpcService>>().await;
	health_reporter.set_serving::<CookieServer<CookieService>>().await;

	let external_addr = cheetah_libraries_microservice::get_external_service_binding_addr();

	let builder = Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(health_service))
		.add_service(tonic_web::enable(token_grpc_service))
		.add_service(tonic_web::enable(cookie_grpc_service));

	builder.serve(external_addr).await.unwrap();
}
