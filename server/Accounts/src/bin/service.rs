use include_dir::include_dir;

use cheetah_accounts::grpc::run_grpc_server;
use cheetah_libraries_ydb::builder::YdbClientBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("accounts");

	let ydb_client = YdbClientBuilder::new_from_env("accounts")
		.prepare_schema_and_build_client(&include_dir!("$CARGO_MANIFEST_DIR/migrations"))
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
