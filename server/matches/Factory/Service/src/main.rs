use std::path::Path;

use tonic::transport::Server;

use cheetah_matches_factory::proto::matches::factory::internal::factory_server::FactoryServer;
use cheetah_matches_factory::service::FactoryService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cheetah_microservice::init("match.factory");
    let templates_path = cheetah_microservice::get_env("TEMPLATES_PATH");
    let registry_grpc_service = format!(
        "{}:{}",
        cheetah_microservice::get_env("REGISTRY_GRPC_SERVICE_HOST"),
        cheetah_microservice::get_env("REGISTRY_GRPC_SERVICE_PORT")
    );
    let service = FactoryService::new(registry_grpc_service, Path::new(&templates_path));
    Server::builder()
        .add_service(FactoryServer::new(service))
        .serve(cheetah_microservice::get_internal_grpc_address())
        .await
        .unwrap();
    Result::Ok(())
}
