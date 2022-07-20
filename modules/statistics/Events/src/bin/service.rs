extern crate core;

use tonic::transport::Server;
use tonic_health::ServingStatus;

use cheetah_statistics_events::proto;
use cheetah_statistics_events::service::EventsService;
use proto::events_server::EventsServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("statistics-event-receiver");
	let loki_url = cheetah_libraries_microservice::get_env("LOKI_URL");
	let namespace = cheetah_libraries_microservice::get_env("NAMESPACE");
	run_grpc_server(loki_url.as_str(), namespace.as_str()).await;
	Ok(())
}

pub async fn run_grpc_server(loki_url: &str, namespace: &str) {
	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

	health_reporter
		.set_service_status("", ServingStatus::Serving)
		.await;
	let grpc_service = EventsServer::new(EventsService::new(loki_url, namespace));
	// если мы здесь - то соединение к базе установлены, все параметры заданы
	// то есть мы можем сказать что сервисы тоже готовы
	health_reporter
		.set_serving::<EventsServer<EventsService>>()
		.await;

	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(health_service))
		.add_service(tonic_web::enable(grpc_service))
		.serve(cheetah_libraries_microservice::get_external_service_binding_addr())
		.await
		.unwrap();
}
