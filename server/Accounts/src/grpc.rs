use jsonwebtoken_google::Parser;
use tonic::transport::Server;

use cheetah_microservice::jwt::JWTTokenParser;

use crate::cookie::CookieGrpcService;
use crate::google::storage::GoogleStorage;
use crate::google::GoogleGrpcService;
use crate::tokens::grpc::TokensGrpcService;
use crate::tokens::TokensService;
use crate::users::UserService;
use crate::{proto, PgPool};

pub async fn run_grpc_server(
	jwt_public_key: String,
	jwt_private_key: String,
	redis_host: &str,
	redis_port: u16,
	redis_auth: Option<String>,
	pool: PgPool,
	google_client_id: Option<String>,
) {
	let token_service = TokensService::new(jwt_private_key, jwt_public_key.clone(), redis_host, redis_port, redis_auth);
	let user_service = UserService::new(pool.clone());

	let token_grpc_service = proto::tokens_server::TokensServer::new(TokensGrpcService::new(token_service.clone()));
	let cookie_grpc_service = proto::cookie_server::CookieServer::new(CookieGrpcService::new(
		pool.clone(),
		token_service.clone(),
		user_service.clone(),
	));

	let external_addr = cheetah_microservice::get_external_service_binding_addr();

	let builder = Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(token_grpc_service))
		.add_service(tonic_web::enable(cookie_grpc_service));

	if let Some(google_client_id) = google_client_id {
		let google_grpc_service = proto::google_server::GoogleServer::new(GoogleGrpcService::new(
			GoogleStorage::new(pool),
			token_service.clone(),
			user_service.clone(),
			Parser::new(&google_client_id),
			JWTTokenParser::new(jwt_public_key),
		));
		let builder = builder.add_service(tonic_web::enable(google_grpc_service));
		builder.serve(external_addr).await.unwrap();
	} else {
		builder.serve(external_addr).await.unwrap();
	}
}
