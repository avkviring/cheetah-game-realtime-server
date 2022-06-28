use cheetah_libraries_microservice::get_env;
use cheetah_libraries_ydb::builder::YdbClientBuilder;
use cheetah_userstore::Service;
use include_dir::include_dir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let jwt_public_key = get_env("JWT_PUBLIC_KEY");

	let ydb_client = YdbClientBuilder::new_from_env("userstore")
		.prepare_schema_and_build_client(&include_dir!("$CARGO_MANIFEST_DIR/migrations"))
		.await;

	let service = Service::new(ydb_client, jwt_public_key);

	let addr = cheetah_libraries_microservice::get_external_service_binding_addr();

	service.serve(addr).await?;

	Ok(())
}
