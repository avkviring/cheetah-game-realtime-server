use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::Arc;

use futures::join;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic_health::ServingStatus;

use crate::agones::run_agones_sdk;
use crate::debug::dump::DumpGrpcService;
use crate::debug::grpc::RealtimeAdminGRPCService;
use crate::debug::proto::admin;
use crate::debug::tracer::grpc::CommandTracerGRPCService;
use crate::grpc::RealtimeInternalService;
use crate::server::manager::RoomsServerManager;

pub mod agones;
pub mod debug;
pub mod grpc;
pub mod room;
pub mod server;

///
/// Паттерн Создатель для игрового сервера
/// - если адреса для udp/grpc не заданы - то в качестве адреса выбирается 127.0.0.1, в качестве
/// порта - свободный порт
///
pub struct ServerBuilder {
	game_bind_addr: SocketAddr,
	admin_grpc_bind_addr: SocketAddr,
	internal_grpc_bind_addr: SocketAddr,
	is_agones_enabled: bool,
}

///
/// Server = Agones + Grpc + NetworkRoomsServerManager
/// NetworkRoomsServerManager = Manager + NetworkRoomsServer
/// где
///  Server - композиция из grpc и
///
pub struct Server {
	pub game_socket_addr: SocketAddr,
	pub manager: Arc<Mutex<RoomsServerManager>>,
	pub internal_grpc_tcp_listener: TcpListener,
	pub admin_grpc_tcp_listener: TcpListener,
	pub is_agones_enabled: bool,
}

impl Default for ServerBuilder {
	fn default() -> Self {
		Self {
			game_bind_addr: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			admin_grpc_bind_addr: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			internal_grpc_bind_addr: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			is_agones_enabled: false,
		}
	}
}

impl Server {
	pub async fn run(self) {
		let internal_grpc = Self::configure_internal_grpc_service(self.internal_grpc_tcp_listener, self.manager.clone());
		let admin_grpc = Self::configure_admin_grpc_service(self.admin_grpc_tcp_listener, self.manager.clone());
		if self.is_agones_enabled {
			let agones = run_agones_sdk(self.manager.clone());
			join!(internal_grpc, admin_grpc, agones);
		} else {
			join!(internal_grpc, admin_grpc);
		}
	}

	async fn configure_internal_grpc_service(tcp_listener: TcpListener, manager: Arc<Mutex<RoomsServerManager>>) {
		let service = grpc::proto::internal::realtime_server::RealtimeServer::new(RealtimeInternalService::new(manager));

		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
		health_reporter.set_service_status("", ServingStatus::Serving).await;

		tonic::transport::server::Server::builder()
			.add_service(health_service)
			.add_service(service)
			.serve_with_incoming(TcpListenerStream::new(tcp_listener))
			.await
			.unwrap()
	}

	async fn configure_admin_grpc_service(tcp_listener: TcpListener, manager: Arc<Mutex<RoomsServerManager>>) {
		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
		health_reporter.set_service_status("", ServingStatus::Serving).await;

		let admin = admin::realtime_server::RealtimeServer::new(RealtimeAdminGRPCService::new(manager.clone()));
		let tracer = admin::command_tracer_server::CommandTracerServer::new(CommandTracerGRPCService::new(manager.clone()));
		let dumper = admin::dump_server::DumpServer::new(DumpGrpcService::new(manager));

		tonic::transport::Server::builder()
			.accept_http1(true)
			.add_service(tonic_web::enable(health_service))
			.add_service(tonic_web::enable(dumper))
			.add_service(tonic_web::enable(admin))
			.add_service(tonic_web::enable(tracer))
			.serve_with_incoming(TcpListenerStream::new(tcp_listener))
			.await
			.unwrap()
	}
}

impl ServerBuilder {
	pub fn set_game_address(mut self, addr: SocketAddr) -> Self {
		self.game_bind_addr = addr;
		self
	}

	pub fn set_admin_grpc_address(mut self, addr: SocketAddr) -> Self {
		self.admin_grpc_bind_addr = addr;
		self
	}

	pub fn set_internal_grpc_address(mut self, addr: SocketAddr) -> Self {
		self.internal_grpc_bind_addr = addr;
		self
	}

	pub fn enable_agones(mut self) -> Self {
		self.is_agones_enabled = true;
		self
	}

	pub async fn build(self) -> Server {
		let game_socket = UdpSocket::bind(self.game_bind_addr).unwrap();
		let game_socket_addr = game_socket.local_addr().unwrap();
		let manager = Arc::new(Mutex::new(RoomsServerManager::new(game_socket)));

		let internal_grpc_bind_listener = TcpListener::bind(self.internal_grpc_bind_addr).await.unwrap();
		let admin_grpc_bind_listener = TcpListener::bind(self.admin_grpc_bind_addr).await.unwrap();

		Server {
			game_socket_addr,
			internal_grpc_tcp_listener: internal_grpc_bind_listener,
			admin_grpc_tcp_listener: admin_grpc_bind_listener,
			is_agones_enabled: self.is_agones_enabled,
			manager,
		}
	}
}
