use cheetah_server::builder::ServerBuilder;
use cheetah_server::env::{
	get_debug_rest_service_default_address, get_env_or_default, get_internal_grpc_service_default_address, get_internal_webgrpc_service_default_address, setup_panic_hook, setup_tracer,
};
use std::str::FromStr;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	setup_tracer(tracing::Level::from_str(get_env_or_default("LOG_LEVEL", "info").as_str()).unwrap());
	setup_panic_hook();
	prometheus_measures_exporter::start_prometheus_exporter();
	tracing::info!("start server");

	let mut builder = ServerBuilder::default()
		.set_internal_grpc_service_bind_address(get_internal_grpc_service_default_address())
		.set_internal_webgrpc_service_bind_address(get_internal_webgrpc_service_default_address())
		.set_debug_rest_service_bind_address(get_debug_rest_service_default_address())
		.set_games_service_bind_address("0.0.0.0:5555".parse().unwrap())
		.set_disconnect_duration(Duration::from_secs(get_env_or_default("DISCONNECT_TIMEOUT_IN_SEC", "180").parse().unwrap()));

	if std::env::var("ENABLE_AGONES").is_ok() {
		builder = builder.enable_agones();
	}

	let server = builder.build().await.unwrap();
	server.run().await;

	Ok(())
}
