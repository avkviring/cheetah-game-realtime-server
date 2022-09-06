use std::path::PathBuf;

use tonic::transport::Server;
use tonic_health::ServingStatus;

use cheetah_libraries_microservice::tonic::codegen::Future;
use cheetah_libraries_microservice::tonic::transport::Error;
use cheetah_matches_factory::proto::matches::factory::admin::configurations_server::ConfigurationsServer;
use cheetah_matches_factory::proto::matches::factory::internal::factory_server::FactoryServer;
use cheetah_matches_factory::service::admin::ConfigurationsService;
use cheetah_matches_factory::service::configuration::yaml::YamlConfigurations;
use cheetah_matches_factory::service::grpc::registry_client::RegistryClient;
use cheetah_matches_factory::service::FactoryService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("matches.factory");
	let templates_path = cheetah_libraries_microservice::get_env("TEMPLATES_PATH");
	let configurations = YamlConfigurations::load(PathBuf::from(templates_path))?;

	let internal_server = create_internal_grpc_server(&configurations).await;
	let admin_server = create_admin_grpc_server(&configurations).await;
	let (res1, res2) = futures::join!(internal_server, admin_server);
	res1.unwrap();
	res2.unwrap();
	Ok(())
}

async fn create_admin_grpc_server(configurations: &YamlConfigurations) -> impl Future<Output = Result<(), Error>> {
	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
	health_reporter.set_service_status("", ServingStatus::Serving).await;

	let service = ConfigurationsService::new(configurations);
	let grpc_service = ConfigurationsServer::new(service);
	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(health_service))
		.add_service(tonic_web::enable(grpc_service))
		.serve(cheetah_libraries_microservice::get_admin_service_binding_addr())
}
async fn create_internal_grpc_server(configurations: &YamlConfigurations) -> impl Future<Output = Result<(), Error>> {
	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
	health_reporter.set_service_status("", ServingStatus::Serving).await;

	let registry_url = cheetah_libraries_microservice::get_internal_srv_uri_from_env("CHEETAH_MATCHES_REGISTRY");
	let registry = RegistryClient::new(registry_url.clone())
		.await
		.unwrap_or_else(|_| panic!("Can not connect to {:?}", registry_url));
	let service = FactoryService::new(registry, configurations).unwrap();
	Server::builder()
		.add_service(health_service)
		.add_service(FactoryServer::new(service))
		.serve(cheetah_libraries_microservice::get_internal_service_binding_addr())
}
