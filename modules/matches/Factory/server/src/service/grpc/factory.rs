use lazy_static::lazy_static;
use prometheus::{register_int_counter, IntCounter};

use cheetah_libraries_microservice::tonic::{Request, Response, Status};

use crate::proto::matches::factory::internal as factory;
use crate::service::FactoryService;

#[tonic::async_trait]
impl factory::factory_server::Factory for FactoryService {
	async fn create_match(
		&self,
		request: Request<factory::CreateMatchRequest>,
	) -> Result<Response<factory::CreateMatchResponse>, Status> {
		self.do_create_match(request.into_inner().template)
			.await
			.map(Response::new)
	}
}

lazy_static! {
	static ref CREATE_MATCH_COUNTER: IntCounter =
		register_int_counter!("create_match_counter", "").unwrap();
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use tokio::net::TcpListener;
	use tokio_stream::wrappers::ReceiverStream;
	use tonic::transport::{Server, Uri};
	use tonic::{Request, Response, Status};

	use realtime::internal;

	use crate::proto::matches::realtime::internal::{
		EmptyRequest, ProbeRequest, ProbeResponse, RoomIdResponse,
	};
	use crate::proto::matches::registry::internal::{Addr, RelayAddrs};
	use crate::proto::matches::{realtime, registry};
	use crate::service::configuration::yaml::test::EXAMPLE_DIR;
	use crate::service::configuration::yaml::YamlConfigurations;
	use crate::service::grpc::registry_client::RegistryClient;
	use crate::service::FactoryService;

	struct StubRegistry {
		addrs: RelayAddrs,
	}

	#[tonic::async_trait]
	impl registry::internal::registry_server::Registry for StubRegistry {
		async fn find_free_relay(
			&self,
			_request: Request<registry::internal::FindFreeRelayRequest>,
		) -> Result<Response<registry::internal::FindFreeRelayResponse>, Status> {
			Ok(Response::new(registry::internal::FindFreeRelayResponse {
				addrs: Some(self.addrs.clone()),
			}))
		}

		async fn update_relay_status(
			&self,
			_: Request<registry::internal::RelayStatusUpdate>,
		) -> Result<Response<registry::internal::UpdateRelayStatusResponse>, Status> {
			unimplemented!()
		}
	}

	struct StubRealtimeService {}
	impl StubRealtimeService {
		pub const ROOM_ID: u64 = 555;
	}
	#[tonic::async_trait]
	impl internal::realtime_server::Realtime for StubRealtimeService {
		async fn create_room(
			&self,
			_request: Request<internal::RoomTemplate>,
		) -> Result<Response<RoomIdResponse>, Status> {
			Ok(Response::new(RoomIdResponse {
				room_id: StubRealtimeService::ROOM_ID,
			}))
		}

		async fn create_member(
			&self,
			_request: Request<internal::CreateMemberRequest>,
		) -> Result<Response<internal::CreateMemberResponse>, Status> {
			unimplemented!()
		}

		async fn create_super_member(
			&self,
			_request: Request<internal::CreateSuperMemberRequest>,
		) -> Result<Response<internal::CreateMemberResponse>, Status> {
			unimplemented!()
		}

		async fn probe(
			&self,
			_request: Request<ProbeRequest>,
		) -> Result<Response<ProbeResponse>, Status> {
			Ok(Response::new(ProbeResponse {}))
		}

		type WatchCreatedRoomEventStream = ReceiverStream<Result<RoomIdResponse, Status>>;

		async fn watch_created_room_event(
			&self,
			_request: Request<EmptyRequest>,
		) -> Result<Response<Self::WatchCreatedRoomEventStream>, Status> {
			todo!()
		}
	}

	#[tokio::test]
	async fn should_create_relay_room() {
		let templates_directory = prepare_templates();
		let uri = stub_grpc_services().await;

		let registry = RegistryClient::new(uri).await.unwrap();
		let factory = FactoryService::new(
			registry,
			&YamlConfigurations::load(templates_directory).unwrap(),
		)
		.unwrap();
		let result = factory.do_create_match("gubaha".to_string()).await.unwrap();
		assert_eq!(result.id, StubRealtimeService::ROOM_ID);
	}

	async fn stub_grpc_services() -> Uri {
		let stub_grpc_service_tcp = TcpListener::bind("127.0.0.1:0").await.unwrap();
		let stub_grpc_service_addr = stub_grpc_service_tcp.local_addr().unwrap();

		let stub_registry = StubRegistry {
			addrs: RelayAddrs {
				// not used
				game: Some(Addr {
					host: "127.0.0.1".to_string(),
					port: 0,
				}),
				grpc_internal: Some(Addr {
					host: stub_grpc_service_addr.ip().to_string(),
					port: stub_grpc_service_addr.port() as u32,
				}),
			},
		};

		let stub_relay = StubRealtimeService {};
		tokio::spawn(async move {
			Server::builder()
				.add_service(registry::internal::registry_server::RegistryServer::new(
					stub_registry,
				))
				.add_service(internal::realtime_server::RealtimeServer::new(stub_relay))
				.serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(
					stub_grpc_service_tcp,
				))
				.await
		});

		cheetah_libraries_microservice::make_internal_srv_uri(
			&stub_grpc_service_addr.ip().to_string(),
			stub_grpc_service_addr.port(),
		)
	}

	// Подготовка шаблонов в каталоге
	fn prepare_templates() -> PathBuf {
		let temp_dir = tempfile::tempdir().unwrap();
		let path = temp_dir.into_path();
		EXAMPLE_DIR.extract(path.clone()).unwrap();
		path
	}
}
