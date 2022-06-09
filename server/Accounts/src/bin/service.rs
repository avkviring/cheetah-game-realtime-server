use cheetah_accounts::grpc::run_grpc_server;
use cheetah_accounts::ydb::connect_to_ydb_and_prepare_schema;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("accounts");

	// ключи для генерации токенов
	let jwt_public_key = cheetah_libraries_microservice::get_env("JWT_PUBLIC_KEY");
	let jwt_private_key = cheetah_libraries_microservice::get_env("JWT_PRIVATE_KEY");

	// параметры ydb
	let ydb_host = cheetah_libraries_microservice::get_env("YDB_HOST");
	let ydb_port: u16 = cheetah_libraries_microservice::get_env("YDB_PORT").parse().unwrap();
	let ydb_client = connect_to_ydb_and_prepare_schema("accounts", &ydb_host, ydb_port).await;

	let google_client_id = std::env::var("AUTH_GOOGLE_CLIENT_ID").ok();

	run_grpc_server(jwt_public_key, jwt_private_key, ydb_client.table_client(), google_client_id).await;
	Ok(())
}
