use tonic::transport::Server;
use tonic::{Request, Response, Status};

use cheetah_matches_stub_registry::proto::internal::registry_server::Registry;
use cheetah_matches_stub_registry::proto::internal::{Addr, FindFreeRelayResponse, RelayAddrs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("matches.stub-registry");

	let registry_service = RegistryService {
		relay_grpc_host: cheetah_libraries_microservice::get_env(
			"MATCHES_RELAY_INTERNAL_GRPC_HOST",
		),
		relay_grpc_port: cheetah_libraries_microservice::get_env(
			"MATCHES_RELAY_INTERNAL_GRPC_PORT",
		)
		.parse()
		.unwrap(),
		relay_game_host: cheetah_libraries_microservice::get_env(
			"MATCHES_RELAY_EXTERNAL_GAME_HOST",
		),
		relay_game_port: cheetah_libraries_microservice::get_env(
			"MATCHES_RELAY_EXTERNAL_GAME_PORT",
		)
		.parse()
		.unwrap(),
	};

	let grpc_service =
		cheetah_matches_stub_registry::proto::internal::registry_server::RegistryServer::new(
			registry_service,
		);
	Server::builder()
		.add_service(grpc_service)
		.serve(cheetah_libraries_microservice::get_internal_service_binding_addr())
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
		_request: Request<cheetah_matches_stub_registry::proto::internal::FindFreeRelayRequest>,
	) -> Result<Response<FindFreeRelayResponse>, Status> {
		let addrs = RelayAddrs {
			game: Some(Addr {
				host: self.relay_game_host.clone(),
				port: self.relay_game_port as u32,
			}),
			grpc_internal: Some(Addr {
				host: self.relay_grpc_host.clone(),
				port: self.relay_grpc_port as u32,
			}),
		};

		Result::Ok(Response::new(FindFreeRelayResponse { addrs: Some(addrs) }))
	}

	async fn update_relay_status(
		&self,
		_request: tonic::Request<cheetah_matches_stub_registry::proto::internal::RelayStatusUpdate>,
	) -> Result<
		tonic::Response<cheetah_matches_stub_registry::proto::internal::UpdateRelayStatusResponse>,
		Status,
	> {
		Ok(tonic::Response::new(
			cheetah_matches_stub_registry::proto::internal::UpdateRelayStatusResponse::default(),
		))
	}
}
