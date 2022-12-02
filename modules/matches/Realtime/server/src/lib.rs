use std::net::SocketAddr;
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
use crate::server::manager::{RoomsServerManager, RoomsServerManagerError};

pub mod agones;
pub mod builder;
pub mod debug;
pub mod grpc;
pub mod room;
pub mod server;

///
/// Server = Agones + Grpc + `NetworkRoomsServerManager`
/// `NetworkRoomsServerManager` = Manager + `NetworkRoomsServer`
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

impl Server {
	pub async fn run(self) {
		let internal_grpc = Self::configure_internal_grpc_service(self.internal_grpc_tcp_listener, Arc::clone(&self.manager));
		let admin_grpc = Self::configure_admin_grpc_service(self.admin_grpc_tcp_listener, Arc::clone(&self.manager));
		if self.is_agones_enabled {
			let agones = run_agones_sdk(Arc::clone(&self.manager));
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
			.unwrap();
	}

	async fn configure_admin_grpc_service(tcp_listener: TcpListener, manager: Arc<Mutex<RoomsServerManager>>) {
		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
		health_reporter.set_service_status("", ServingStatus::Serving).await;

		let admin = admin::realtime_server::RealtimeServer::new(RealtimeAdminGRPCService::new(Arc::clone(&manager)));
		let tracer = admin::command_tracer_server::CommandTracerServer::new(CommandTracerGRPCService::new(Arc::clone(&manager)));
		let dumper = admin::dump_server::DumpServer::new(DumpGrpcService::new(manager));

		tonic::transport::Server::builder()
			.accept_http1(true)
			.add_service(tonic_web::enable(health_service))
			.add_service(tonic_web::enable(dumper))
			.add_service(tonic_web::enable(admin))
			.add_service(tonic_web::enable(tracer))
			.serve_with_incoming(TcpListenerStream::new(tcp_listener))
			.await
			.unwrap();
	}
}
