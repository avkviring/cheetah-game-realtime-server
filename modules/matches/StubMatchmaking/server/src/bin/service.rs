use tonic::transport::Server;

use cheetah_libraries_microservice::auth::JwtAuthInterceptor;

use cheetah_matches_stub_matchmaking::configuration::YamlConfig;
use cheetah_matches_stub_matchmaking::proto::matches::matchmaking;
use cheetah_matches_stub_matchmaking::service::StubMatchmakingService;
use matchmaking::external::matchmaking_server::MatchmakingServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("matches.stubmatchmaking");

	let factory_url = cheetah_libraries_microservice::get_internal_srv_uri_from_env("CHEETAH_MATCHES_FACTORY");
	let jwt_public_key = cheetah_libraries_microservice::get_env("JWT_PUBLIC_KEY");
	let config_file = cheetah_libraries_microservice::get_env("CONFIG_FILE");
	let rulemap = YamlConfig::from_file(config_file.into()).unwrap().rulemap();

	let service = StubMatchmakingService::new(factory_url, rulemap);

	let interceptor = JwtAuthInterceptor::new(jwt_public_key);
	let service = MatchmakingServer::with_interceptor(service, interceptor);

	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(service))
		.serve(cheetah_libraries_microservice::get_external_service_binding_addr())
		.await
		.unwrap();

	Result::Ok(())
}
