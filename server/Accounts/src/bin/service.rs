use include_dir::include_dir;

use cheetah_accounts::grpc::run_grpc_server;
use cheetah_libraries_ydb::connect_to_ydb_and_prepare_schema;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("accounts");

	// параметры ydb
	let ydb_namespace =
		cheetah_libraries_microservice::get_env_or_default("YDB_NAMESPACE", "local");
	let ydb_host = cheetah_libraries_microservice::get_env("YDB_HOST");
	let ydb_port: u16 = cheetah_libraries_microservice::get_env("YDB_PORT")
		.parse()
		.unwrap_or_else(|_| {
			panic!(
				"Expect number but {}",
				cheetah_libraries_microservice::get_env("YDB_PORT")
			)
		});

	let ydb_client = connect_to_ydb_and_prepare_schema(
		ydb_namespace.as_str(),
		"accounts",
		ydb_host.as_str(),
		ydb_port,
		&include_dir!("$CARGO_MANIFEST_DIR/migrations"),
	)
	.await;

	// ключи для генерации токенов
	let jwt_public_key = cheetah_libraries_microservice::get_env("JWT_PUBLIC_KEY");
	let jwt_private_key = cheetah_libraries_microservice::get_env("JWT_PRIVATE_KEY");
	let google_client_id = std::env::var("AUTH_GOOGLE_CLIENT_ID").ok();

	run_grpc_server(
		jwt_public_key,
		jwt_private_key,
		ydb_client.table_client(),
		google_client_id,
	)
	.await;
	Ok(())
}
