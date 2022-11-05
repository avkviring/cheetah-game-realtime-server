use tonic::transport::Server;

use cheetah_matches_stub_matchmaking::proto::matches::matchmaking;
use cheetah_matches_stub_matchmaking::service::StubMatchmakingService;
use matchmaking::external::matchmaking_server::MatchmakingServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("matches.stubmatchmaking");

	let factory_url = cheetah_libraries_microservice::get_internal_srv_uri_from_env("CHEETAH_MATCHES_FACTORY");

	let service = StubMatchmakingService::new(factory_url);

	let service = MatchmakingServer::new(service);

	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(service))
		.serve(cheetah_libraries_microservice::get_external_service_binding_addr())
		.await
		.unwrap();

	Result::Ok(())
}
