use cheetah_libraries_microservice::get_env;
use cheetah_libraries_postgresql::create_postgresql_pool_from_env;
use cheetah_user_store::grpc::Service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("user.store");
	let jwt_public_key = get_env("JWT_PUBLIC_KEY");
	let pg_pool = create_postgresql_pool_from_env().await;
	sqlx::migrate!().run(&pg_pool).await.unwrap();
	let service = Service::new(pg_pool, jwt_public_key);
	let addr = cheetah_libraries_microservice::get_external_service_binding_addr();
	service.serve(addr).await?;
	Ok(())
}
