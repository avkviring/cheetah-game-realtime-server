use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use futures::join;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic_health::ServingStatus;
use tonic_web::GrpcWebLayer;

use crate::agones::agones_und_notifyservice_cycle;
use crate::grpc::proto::internal::internal_server::InternalServer;
use crate::grpc::RealtimeInternalService;
use crate::rest::run_rest_server;
use crate::server::manager::{RoomsServerManagerError, ServerManager};

pub mod agones;
pub mod builder;
pub mod env;
pub mod grpc;
pub mod rest;
pub mod room;
pub mod server;

///
/// Server = Agones + Grpc + `NetworkRoomsServerManager`
/// `NetworkRoomsServerManager` = Manager + `NetworkRoomsServer`
///
pub struct Server {
	pub game_socket_addr: SocketAddr,
	pub internal_webgrpc_listener: TcpListener,
	pub internal_grpc_listener: TcpListener,
	pub debug_rest_service_listener: TcpListener,
	pub is_agones_enabled: bool,
	pub manager: Arc<Mutex<ServerManager>>,
}

impl Server {
	pub async fn run(self) {
		let internal_grpc_future = Self::new_internal_grpc_service(self.internal_grpc_listener, Arc::clone(&self.manager));
		let internal_webgrpc_future = Self::new_internal_webgrpc_service(self.internal_webgrpc_listener, Arc::clone(&self.manager));
		let debug_rest_service = run_rest_server(Arc::clone(&self.manager), self.debug_rest_service_listener);
		if self.is_agones_enabled {
			let max_rooms = usize::from_str(&env::get_env_or_default("MAX_ROOMS", "20")).unwrap();
			let agones = agones_und_notifyservice_cycle(Arc::clone(&self.manager), max_rooms);
			join!(internal_grpc_future, internal_webgrpc_future, debug_rest_service, agones);
		} else {
			join!(internal_grpc_future, internal_webgrpc_future, debug_rest_service);
		}
	}

	async fn new_internal_grpc_service(listener: TcpListener, manager: Arc<Mutex<ServerManager>>) {
		let service = InternalServer::new(RealtimeInternalService::new(manager));

		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
		health_reporter.set_service_status("", ServingStatus::Serving).await;

		tonic::transport::server::Server::builder()
			.add_service(health_service.clone())
			.add_service(service.clone())
			.serve_with_incoming(TcpListenerStream::new(listener))
			.await
			.unwrap();
	}

	async fn new_internal_webgrpc_service(listener: TcpListener, manager: Arc<Mutex<ServerManager>>) {
		let service = InternalServer::new(RealtimeInternalService::new(manager));

		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
		health_reporter.set_service_status("", ServingStatus::Serving).await;

		tonic::transport::server::Server::builder()
			.accept_http1(true)
			.layer(GrpcWebLayer::new())
			.add_service(health_service)
			.add_service(service)
			.serve_with_incoming(TcpListenerStream::new(listener))
			.await
			.unwrap();
	}
}
