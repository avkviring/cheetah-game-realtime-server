use cheetah_libraries_microservice::get_env;
use cheetah_userstore::Service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let jwt_public_key = get_env("JWT_PUBLIC_KEY");

	let service = Service::new(ydb_client, jwt_public_key);

	Ok(())
}
