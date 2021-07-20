use std::path::Path;
use std::str::FromStr;

use tonic::transport::{Server, Uri};

use cheetah_matches_factory::proto::matches::factory::internal::factory_server::FactoryServer;
use cheetah_matches_factory::service::FactoryService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cheetah_microservice::init("match.factory");
    let templates_path = cheetah_microservice::get_env("TEMPLATES_PATH");
    let registry_grpc_service = cheetah_microservice::make_internal_grpc_uri(
        cheetah_microservice::get_env("REGISTRY_GRPC_INTERNAL_SERVICE_HOST").as_str(),
        cheetah_microservice::get_env("REGISTRY_GRPC_INTERNAL_SERVICE_PORT")
            .parse()
            .unwrap(),
    );
    let service = FactoryService::new(registry_grpc_service, Path::new(&templates_path)).unwrap();
    Server::builder()
        .add_service(FactoryServer::new(service))
        .serve(cheetah_microservice::get_self_service_internal_grpc_address())
        .await
        .unwrap();
    Result::Ok(())
}
