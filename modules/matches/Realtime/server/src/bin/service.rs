use cheetah_matches_realtime::builder::ServerBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("matches.relay");

	let mut builder = ServerBuilder::default()
		.set_admin_grpc_address(cheetah_libraries_microservice::get_admin_service_binding_addr())
		.set_internal_grpc_address(cheetah_libraries_microservice::get_internal_service_binding_addr())
		.set_game_address("0.0.0.0:5555".parse().unwrap());

	if std::env::var("ENABLE_AGONES").is_ok() {
		builder = builder.enable_agones();
	}

	let server = builder.build().await.unwrap();
	server.run().await;

	Ok(())
}
