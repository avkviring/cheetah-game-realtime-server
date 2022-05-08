extern crate core;

use std::fs;

use tonic::transport::Server;
use tonic_health::ServingStatus;

use cheetah_system_compatibility::config::Config;
use cheetah_system_compatibility::proto::compatibility_checker_server::CompatibilityCheckerServer;
use cheetah_system_compatibility::service::Service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("system-compatibility");
	let config_file = cheetah_microservice::get_env("CONFIG_FILE");
	run_grpc_server(config_file.as_str()).await;
	Ok(())
}

pub async fn run_grpc_server(config_file: &str) {
	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

	health_reporter.set_service_status("", ServingStatus::Serving).await;
	let config_content = fs::read_to_string(config_file).unwrap();
	let config = Config::new(config_content).unwrap();
	let grpc_service = CompatibilityCheckerServer::new(Service::new(config.to_versions()));
	health_reporter.set_serving::<CompatibilityCheckerServer<Service>>().await;

	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(health_service))
		.add_service(tonic_web::enable(grpc_service))
		.serve(cheetah_microservice::get_external_service_binding_addr())
		.await
		.unwrap();
}
