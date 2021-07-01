use tonic::transport::Server;

use cheetah_matches_factory::proto::matches::factory::internal::factory_server::FactoryServer;
use cheetah_matches_factory::service::FactoryService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cheetah_microservice::init("MatchFactory");
    // let service = FactoryService::new(cheetah_microservice::get_env("TEMPLATES_PATH"));
    // Server::builder()
    //     .add_service(FactoryServer::new(service))
    //     .serve(cheetah_microservice::get_internal_grpc_address())
    //     .await
    //     .unwrap();
    Result::Ok(())
}
