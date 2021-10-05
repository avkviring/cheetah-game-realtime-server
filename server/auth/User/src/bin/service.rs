use cheetah_auth_user::{run_grpc_server, storage::create_postgres_pool};
use cheetah_microservice::get_env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("auth::User");

	let pg_user = get_env("POSTGRES_USER");
	let pg_passwd = get_env("POSTGRES_PASSWORD");
	let pg_db = get_env("POSTGRES_DB");
	let pg_host = get_env("POSTGRES_HOST");
	let pg_port: u16 = get_env("POSTGRES_PORT").parse().unwrap();

	let pool = create_postgres_pool(&pg_db, &pg_user, &pg_passwd, &pg_host, pg_port).await;

	run_grpc_server(pool, cheetah_microservice::get_internal_service_binding_addr()).await;

	Ok(())
}
