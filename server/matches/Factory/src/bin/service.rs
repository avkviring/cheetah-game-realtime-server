use std::path::{Path, PathBuf};

use tonic::transport::Server;

use cheetah_matches_factory::proto::matches::factory::internal::factory_server::FactoryServer;
use cheetah_matches_factory::service::configurations::Configurations;
use cheetah_matches_factory::service::{grpc::RegistryClient, Service};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("match.factory");
	let templates_path = cheetah_microservice::get_env("TEMPLATES_PATH");
	let registry = cheetah_microservice::get_internal_srv_uri_from_env("CHEETAH_MATCHES_REGISTRY");
	let registry = RegistryClient::new(registry).unwrap();
	let configurations = Configurations::load(PathBuf::from(templates_path))?;
	let service = Service::new(registry, &configurations).unwrap();
	Server::builder()
		.add_service(FactoryServer::new(service))
		.serve(cheetah_microservice::get_internal_service_binding_addr())
		.await
		.unwrap();
	Ok(())
}
