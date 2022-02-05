use cheetah_matches_registry::proto::matches::registry::internal::registry_server::RegistryServer;
use cheetah_matches_registry::registry::service::RegistryService;
use std::env;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("matches.registry");

	let redis_host = cheetah_microservice::get_env("REDIS_HOST");
	let redis_port: u16 = cheetah_microservice::get_env("REDIS_PORT").parse().unwrap();
	let redis_auth = env::var("REDIS_AUTH").ok();
	let redis_dsn = match redis_auth {
		Some(ref password) => {
			format!("redis://:{}@{}:{}", password, redis_host, redis_port)
		}
		None => {
			format!("redis://{}:{}", redis_host, redis_port)
		}
	};

	let registry_service = RegistryService::new(&redis_dsn).await?;
	let grpc_service = RegistryServer::new(registry_service);
	Server::builder()
		.add_service(grpc_service)
		.serve(cheetah_microservice::get_internal_service_binding_addr())
		.await?;
	Result::Ok(())
}
