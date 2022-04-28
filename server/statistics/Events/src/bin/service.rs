extern crate core;

use tonic::transport::Server;
use tonic_health::ServingStatus;

use cheetah_statistics_events::proto::event_receiver_server::EventReceiverServer;
use cheetah_statistics_events::service::EventReceiverService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("statistics-event-receiver");
	if let Ok(loki_url) = std::env::var("LOKI_URL") {
		run_grpc_server(loki_url.as_str()).await;
	} else {
		panic!("env LOKI_URL ist not set")
	}
	Ok(())
}

pub async fn run_grpc_server(loki_url: &str) {
	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

	health_reporter.set_service_status("", ServingStatus::Serving).await;
	let grpc_service = EventReceiverServer::new(EventReceiverService::new(loki_url));

	// если мы здесь - то соединение к базе установлены, все параметры заданы
	// то есть мы можем сказать что сервисы тоже готовы
	health_reporter
		.set_serving::<EventReceiverServer<EventReceiverService>>()
		.await;

	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(health_service))
		.add_service(tonic_web::enable(grpc_service))
		.serve(cheetah_microservice::get_external_service_binding_addr())
		.await
		.unwrap();
}
