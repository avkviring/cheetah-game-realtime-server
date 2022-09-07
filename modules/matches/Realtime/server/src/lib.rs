use std::future::Future;
use std::net::{SocketAddr, UdpSocket};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

use futures::join;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tonic::transport::Error;
use tonic_health::ServingStatus;

use crate::agones::run_agones_cycle;
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
	enable_agones: bool,
}

///
/// Server = Agones + Grpc + NetworkRoomsServerManager
/// NetworkRoomsServerManager = Manager + NetworkRoomsServer
/// где
///  Server - композиция из grpc и
///
pub struct Server {
	pub game_socket_addr: SocketAddr,
	pub admin_grpc_socket_addr: SocketAddr,
	pub internal_grpc_socket_addr: SocketAddr,
	pub manager: Arc<Mutex<RoomsServerManager>>,
	admin_grpc_feature: Pin<Box<dyn Future<Output = Result<(), Error>>>>,
	internal_grpc_feature: Pin<Box<dyn Future<Output = Result<(), Error>>>>,
	agones_feature: Option<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Default for ServerBuilder {
	fn default() -> Self {
		Self {
			game_bind_addr: SocketAddr::from_str("0.0.0.0:0").unwrap(),
			admin_grpc_bind_addr: SocketAddr::from_str("0.0.0.0:0").unwrap(),
			internal_grpc_bind_addr: SocketAddr::from_str("0.0.0.0:0").unwrap(),
			enable_agones: false,
		}
	}
}

impl Server {
	pub async fn run(self) {
		if self.agones_feature.is_some() {
			let (r1, r2, _) = join!(self.admin_grpc_feature, self.internal_grpc_feature, self.agones_feature.unwrap());
			r1.unwrap();
			r2.unwrap();
		} else {
			let (r1, r2) = join!(self.admin_grpc_feature, self.internal_grpc_feature);
			r1.unwrap();
			r2.unwrap();
		}
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
		self.enable_agones = true;
		self
	}

	pub async fn build(self) -> Server {
		let game_socket = UdpSocket::bind(self.game_bind_addr).unwrap();
		let game_socket_addr = game_socket.local_addr().unwrap();
		let manager = Arc::new(Mutex::new(RoomsServerManager::new(game_socket)));
		let (internal_grpc_socket_addr, internal_grpc_feature) = Self::create_internal_grpc(self.internal_grpc_bind_addr, manager.clone()).await;
		let (admin_grpc_socket_addr, admin_grpc_feature) = Self::create_admin_grpc(self.admin_grpc_bind_addr, manager.clone()).await;

		let mut agones_feature: Option<Pin<Box<dyn Future<Output = ()>>>> = None;
		if self.enable_agones {
			agones_feature = Some(Box::pin(run_agones_cycle(manager.clone())));
		}

		Server {
			game_socket_addr,
			admin_grpc_socket_addr,
			internal_grpc_socket_addr,
			manager,
			admin_grpc_feature: Box::pin(admin_grpc_feature),
			internal_grpc_feature: Box::pin(internal_grpc_feature),
			agones_feature,
		}
	}

	async fn create_admin_grpc(addr: SocketAddr, manager: Arc<Mutex<RoomsServerManager>>) -> (SocketAddr, impl Future<Output = Result<(), Error>>) {
		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
		health_reporter.set_service_status("", ServingStatus::Serving).await;

		let admin = admin::realtime_server::RealtimeServer::new(RealtimeAdminGRPCService::new(manager.clone()));
		let tracer = admin::command_tracer_server::CommandTracerServer::new(CommandTracerGRPCService::new(manager.clone()));
		let dumper = admin::dump_server::DumpServer::new(DumpGrpcService::new(manager));

		let listener = TcpListener::bind(addr).await.unwrap();
		let binding_addr = listener.local_addr().unwrap();
		(
			binding_addr,
			tonic::transport::Server::builder()
				.accept_http1(true)
				.add_service(tonic_web::enable(health_service))
				.add_service(tonic_web::enable(dumper))
				.add_service(tonic_web::enable(admin))
				.add_service(tonic_web::enable(tracer))
				.serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener)),
		)
	}

	async fn create_internal_grpc(
		addr: SocketAddr,
		manager: Arc<Mutex<RoomsServerManager>>,
	) -> (SocketAddr, impl Future<Output = Result<(), Error>>) {
		let service = grpc::proto::internal::realtime_server::RealtimeServer::new(RealtimeInternalService::new(manager));

		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
		health_reporter.set_service_status("", ServingStatus::Serving).await;

		let listener = TcpListener::bind(addr).await.unwrap();
		let binding_addr = listener.local_addr().unwrap();

		(
			binding_addr,
			tonic::transport::server::Server::builder()
				.add_service(health_service)
				.add_service(service)
				.serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener)),
		)
	}
}
