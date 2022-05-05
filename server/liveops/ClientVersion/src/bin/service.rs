extern crate core;

use tonic::transport::Server;
use tonic_health::ServingStatus;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("liveops-client-version");
	let config_file = cheetah_microservice::get_env("CONFIG_FILE");
	run_grpc_server(config_file.as_str()).await;
	Ok(())
}

pub async fn run_grpc_server(config_file: &str) {
	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

	// health_reporter.set_service_status("", ServingStatus::Serving).await;
	// let grpc_service = EventsServer::new(EventsService::new(loki_url, namespace));
	// // если мы здесь - то соединение к базе установлены, все параметры заданы
	// // то есть мы можем сказать что сервисы тоже готовы
	// health_reporter.set_serving::<EventsServer<EventsService>>().await;
	//
	// Server::builder()
	// 	.accept_http1(true)
	// 	.add_service(tonic_web::enable(health_service))
	// 	.add_service(tonic_web::enable(grpc_service))
	// 	.serve(cheetah_microservice::get_external_service_binding_addr())
	// 	.await
	// 	.unwrap();
}
