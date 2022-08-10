use cheetah_libraries_postgresql::create_postgresql_pool_from_env;
use cheetah_user_accounts::grpc::run_grpc_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("accounts");
	let pg_pool = create_postgresql_pool_from_env().await;
	sqlx::migrate!().run(&pg_pool).await.unwrap();

	// ключи для генерации токенов
	let jwt_public_key = cheetah_libraries_microservice::get_env("JWT_PUBLIC_KEY");
	let jwt_private_key = cheetah_libraries_microservice::get_env("JWT_PRIVATE_KEY");
	let google_client_id = std::env::var("AUTH_GOOGLE_CLIENT_ID").ok();

	run_grpc_server(jwt_public_key, jwt_private_key, pg_pool, google_client_id).await;
	Ok(())
}
