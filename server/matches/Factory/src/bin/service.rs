use std::path::PathBuf;

use tonic::transport::Server;
use tonic_health::ServingStatus;

use cheetah_matches_factory::proto::matches::factory::admin::configurations_server::ConfigurationsServer;
use cheetah_matches_factory::proto::matches::factory::internal::factory_server::FactoryServer;
use cheetah_matches_factory::service::admin::ConfigurationsService;
use cheetah_matches_factory::service::configurations::Configurations;
use cheetah_matches_factory::service::grpc::registry_client::RegistryClient;
use cheetah_matches_factory::service::FactoryService;
use cheetah_microservice::tonic::codegen::Future;
use cheetah_microservice::tonic::transport::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("matches.factory");
	let templates_path = cheetah_microservice::get_env("TEMPLATES_PATH");
	let configurations = Configurations::load(PathBuf::from(templates_path))?;

	let internal_server = create_internal_grpc_server(&configurations).await;
	let admin_server = create_admin_grpc_server(&configurations).await;
	let (res1, res2) = futures::join!(internal_server, admin_server);
	res1.unwrap();
	res2.unwrap();
	Ok(())
}

async fn create_admin_grpc_server(configurations: &Configurations) -> impl Future<Output = Result<(), Error>> {
	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
	health_reporter.set_service_status("", ServingStatus::Serving).await;

	let service = ConfigurationsService::new(configurations);
	let grpc_service = ConfigurationsServer::new(service);
	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(health_service))
		.add_service(tonic_web::enable(grpc_service))
		.serve(cheetah_microservice::get_admin_service_binding_addr())
}
async fn create_internal_grpc_server(configurations: &Configurations) -> impl Future<Output = Result<(), Error>> {
	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
	health_reporter.set_service_status("", ServingStatus::Serving).await;

	let registry_url = cheetah_microservice::get_internal_srv_uri_from_env("CHEETAH_MATCHES_REGISTRY");
	let registry = RegistryClient::new(registry_url).await.unwrap();
	let service = FactoryService::new(registry, configurations).unwrap();
	Server::builder()
		.add_service(health_service)
		.add_service(FactoryServer::new(service))
		.serve(cheetah_microservice::get_internal_service_binding_addr())
}
