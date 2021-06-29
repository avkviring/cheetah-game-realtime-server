use tonic::transport::Server;
use tonic::{Request, Response, Status};

use proto::internal::registry_server::Registry;
use proto::internal::FindFreeRelayResponse;

pub mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    println!("±± cheetah match stub registry component ±±");

    let registry_service = RegistryService {
        relay_grpc_host: get_env("RELAY_GRPC_HOST"),
        relay_grpc_port: get_env("RELAY_GRPC_PORT").parse().unwrap(),
        relay_game_host: get_env("RELAY_GAME_HOST"),
        relay_game_port: get_env("RELAY_GAME_PORT").parse().unwrap(),
    };

    let grpc_service = proto::internal::registry_server::RegistryServer::new(registry_service);
    Server::builder()
        .add_service(grpc_service)
        .serve("0.0.0.0:5001".parse().unwrap())
        .await
        .unwrap();

    Result::Ok(())
}

pub struct RegistryService {
    pub relay_grpc_host: String,
    pub relay_grpc_port: u16,
    pub relay_game_host: String,
    pub relay_game_port: u16,
}
#[tonic::async_trait]
impl Registry for RegistryService {
    async fn find_free_relay(
        &self,
        _request: Request<proto::internal::FindFreeRelayRequest>,
    ) -> Result<Response<FindFreeRelayResponse>, Status> {
        Result::Ok(Response::new(FindFreeRelayResponse {
            relay_grpc_host: self.relay_grpc_host.clone(),
            relay_grpc_port: self.relay_game_port as u32,
            relay_game_host: self.relay_game_host.clone(),
            relay_game_port: self.relay_game_port as u32,
        }))
    }
}

fn get_env(name: &str) -> String {
    let value = std::env::var(name).unwrap_or_else(|_| panic!("Env {} is not set", name));
    if value.trim().is_empty() {
        panic!("Env {} is empty", name);
    }
    value
}
