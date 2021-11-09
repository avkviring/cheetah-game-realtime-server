use tonic::transport::Server;

use cheetah_matches_stub_matchmaking::proto::matches::matchmaking;
use cheetah_matches_stub_matchmaking::service::StubMatchmakingService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("matches.stubmatchmaking");
	let factory_url = cheetah_microservice::get_internal_srv_uri_from_env("CHEETAH_MATCHES_FACTORY");
	let jwt_public_key = cheetah_microservice::get_env("JWT_PUBLIC_KEY");
	let service = StubMatchmakingService::new(factory_url, jwt_public_key);
	Server::builder()
		.add_service(matchmaking::external::matchmaking_server::MatchmakingServer::new(service))
		.serve(cheetah_microservice::get_external_service_binding_addr())
		.await
		.unwrap();
	Result::Ok(())
}
