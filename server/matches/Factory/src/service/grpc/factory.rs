use cheetah_microservice::tonic::{Request, Response, Status};

use crate::proto::matches::factory::internal as factory;
use crate::proto::matches::relay::internal::relay_client::RelayClient;
use crate::service::FactoryService;

impl FactoryService {
	async fn do_create_match(&self, template_name: String) -> Result<factory::CreateMatchResponse, Status> {
		// получаем шаблон
		let room_template = self
			.template(&template_name)
			.ok_or_else(|| Status::internal(format!("Template {} not found", template_name)))?;

		// ищем свободный relay сервер
		let relay = self.registry.find_free_relay().await;
		let relay_addr = cheetah_microservice::make_internal_srv_uri(&relay.relay_grpc_host, relay.relay_grpc_port as u16);
		log::info!("Connect to relay {}", relay_addr);
		// создаем матч на relay сервере
		let mut connect = RelayClient::connect(relay_addr).await.unwrap();

		// создаем комнату
		Ok(factory::CreateMatchResponse {
			id: connect.create_room(room_template).await?.into_inner().id,
			relay_grpc_host: relay.relay_grpc_host,
			relay_grpc_port: relay.relay_grpc_port,
			relay_game_host: relay.relay_game_host,
			relay_game_port: relay.relay_game_port,
		})
	}
}

#[tonic::async_trait]
impl factory::factory_server::Factory for FactoryService {
	async fn create_match(
		&self,
		request: Request<factory::CreateMatchRequest>,
	) -> Result<Response<factory::CreateMatchResponse>, Status> {
		self.do_create_match(request.into_inner().template).await.map(Response::new)
	}
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use tokio::net::TcpListener;
	use tonic::transport::{Server, Uri};
	use tonic::{Request, Response, Status};

	use crate::proto::matches::registry::internal as registry;
	use crate::proto::matches::relay;
	use crate::service::configurations::test::EXAMPLE_DIR;
	use crate::service::configurations::Configurations;
	use crate::service::grpc::registry_client::RegistryClient;
	use crate::service::FactoryService;

	struct StubRegistry {
		pub relay_grpc_host: String,
		pub relay_grpc_port: u16,
		pub relay_game_host: String,
		pub relay_game_port: u16,
	}

	#[tonic::async_trait]
	impl registry::registry_server::Registry for StubRegistry {
		async fn find_free_relay(
			&self,
			_request: Request<registry::FindFreeRelayRequest>,
		) -> Result<Response<registry::FindFreeRelayResponse>, Status> {
			Ok(Response::new(registry::FindFreeRelayResponse {
				relay_grpc_host: self.relay_grpc_host.clone(),
				relay_grpc_port: self.relay_grpc_port as u32,
				relay_game_host: self.relay_game_host.clone(),
				relay_game_port: self.relay_game_port as u32,
			}))
		}
	}

	struct StubRelay {}
	impl StubRelay {
		pub const ROOM_ID: u64 = 555;
	}
	#[tonic::async_trait]
	impl relay::internal::relay_server::Relay for StubRelay {
		async fn create_room(
			&self,
			_request: Request<relay::internal::RoomTemplate>,
		) -> Result<tonic::Response<relay::internal::CreateRoomResponse>, tonic::Status> {
			Ok(tonic::Response::new(relay::internal::CreateRoomResponse {
				id: StubRelay::ROOM_ID,
			}))
		}

		async fn attach_user(
			&self,
			_request: tonic::Request<relay::internal::AttachUserRequest>,
		) -> Result<tonic::Response<relay::internal::AttachUserResponse>, tonic::Status> {
			unimplemented!()
		}
	}

	#[tokio::test]
	async fn should_create_relay_room() {
		let templates_directory = prepare_templates();
		let uri = stub_grpc_services().await;

		let registry = RegistryClient::new(uri).unwrap();
		let factory = FactoryService::new(registry, &Configurations::load(templates_directory).unwrap()).unwrap();
		let result = factory.do_create_match("gubaha".to_string()).await.unwrap();
		assert_eq!(result.id, StubRelay::ROOM_ID);
	}

	async fn stub_grpc_services() -> Uri {
		let stub_grpc_service_tcp = TcpListener::bind("127.0.0.1:0").await.unwrap();
		let stub_grpc_service_addr = stub_grpc_service_tcp.local_addr().unwrap();

		let stub_registry = StubRegistry {
			relay_grpc_host: stub_grpc_service_addr.ip().to_string(),
			relay_grpc_port: stub_grpc_service_addr.port(),
			relay_game_host: "game-host".to_owned(),
			relay_game_port: 555,
		};

		let stub_relay = StubRelay {};
		tokio::spawn(async move {
			Server::builder()
				.add_service(registry::registry_server::RegistryServer::new(stub_registry))
				.add_service(relay::internal::relay_server::RelayServer::new(stub_relay))
				.serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(stub_grpc_service_tcp))
				.await
		});

		cheetah_microservice::make_internal_srv_uri(&stub_grpc_service_addr.ip().to_string(), stub_grpc_service_addr.port())
	}

	// Подготовка шаблонов в каталоге
	fn prepare_templates() -> PathBuf {
		let temp_dir = tempfile::tempdir().unwrap();
		let path = temp_dir.into_path();
		EXAMPLE_DIR.extract(path.clone()).unwrap();
		path
	}
}
