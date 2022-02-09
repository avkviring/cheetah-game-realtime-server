use cheetah_accounts::grpc::run_grpc_server;
use cheetah_accounts::postgresql::create_postgres_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	pretty_env_logger::init();
	println!("±± cheetah game cerberus component ±±");

	// ключи для генерации токенов
	let jwt_public_key = get_env("JWT_PUBLIC_KEY");
	let jwt_private_key = get_env("JWT_PRIVATE_KEY");

	// параметры redis
	let redis_host = get_env("REDIS_HOST");
	let redis_port = get_env("REDIS_PORT").parse().unwrap();
	let redis_auth = std::env::var("REDIS_AUTH").ok();

	let pg_user = get_env("POSTGRES_USER");
	let pg_passwd = get_env("POSTGRES_PASSWORD");
	let pg_db = get_env("POSTGRES_DB");
	let pg_host = get_env("POSTGRES_HOST");
	let pg_port: u16 = get_env("POSTGRES_PORT").parse().unwrap();
	let google_client_id = std::env::var("AUTH_GOOGLE_CLIENT_ID").ok();
	let pool = create_postgres_pool(&pg_db, &pg_user, &pg_passwd, &pg_host, pg_port).await;
	run_grpc_server(
		jwt_public_key,
		jwt_private_key,
		&redis_host,
		redis_port,
		redis_auth,
		pool,
		google_client_id,
	)
	.await;
	Ok(())
}
fn get_env(name: &str) -> String {
	let value = std::env::var(name).unwrap_or_else(|_| panic!("Env {}", name));
	if value.trim().is_empty() {
		panic!("Env {} is empty", name);
	}
	value
}
