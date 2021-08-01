use std::str::FromStr;

use tonic::transport::{Server, Uri};

use cheetah_matches_stub_matchmaking::proto::matches::matchmaking;
use cheetah_matches_stub_matchmaking::service::StubMatchmakingService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cheetah_microservice::init("matches.stubmatchmaking");
    let factory_grpc_service = format!(
        "{}:{}",
        cheetah_microservice::get_env("MATCHES_FACTORY_INTERNAL_GRPC_HOST"),
        cheetah_microservice::get_env("MATCHES_FACTORY_INTERNAL_GRPC_PORT")
    );
    let service =
        StubMatchmakingService::new(Uri::from_str(factory_grpc_service.as_str()).unwrap());
    Server::builder()
        .add_service(matchmaking::external::matchmaking_server::MatchmakingServer::new(service))
        .serve(cheetah_microservice::get_internal_service_binding_addr())
        .await
        .unwrap();
    Result::Ok(())
}
