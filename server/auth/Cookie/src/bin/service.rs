use cheetah_auth_cookie::{run_grpc_server, storage};
use cheetah_microservice::get_env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("auth::Cookie");

	let pg_user = get_env("POSTGRES_USER");
	let pg_passwd = get_env("POSTGRES_PASSWORD");
	let pg_db = get_env("POSTGRES_DB");
	let pg_host = get_env("POSTGRES_HOST");
	let pg_port: u16 = get_env("POSTGRES_PORT").parse().unwrap();

	let cerberus_internal_service_uri = cheetah_microservice::get_internal_srv_uri_from_env("CHEETAH_AUTH_CERBERUS");
	let user_internal_service_uri = cheetah_microservice::get_internal_srv_uri_from_env("CHEETAH_AUTH_USER");

	let pool = storage::create_postgres_pool(&pg_db, &pg_user, &pg_passwd, &pg_host, pg_port).await;
	storage::migrate_db(&pool).await;

	run_grpc_server(
		pool,
		cerberus_internal_service_uri,
		user_internal_service_uri,
		cheetah_microservice::get_external_service_binding_addr(),
	)
	.await;

	Ok(())
}
