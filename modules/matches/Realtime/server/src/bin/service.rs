use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::Future;
use tokio::sync::Mutex;
use tonic::transport::Server;
use tonic_health::ServingStatus;

use cheetah_matches_realtime::agones::run_agones_cycle;
use cheetah_matches_realtime::debug::dump::DumpGrpcService;
use cheetah_matches_realtime::debug::grpc::RealtimeAdminGRPCService;
use cheetah_matches_realtime::debug::proto::admin;
use cheetah_matches_realtime::debug::tracer::grpc::CommandTracerGRPCService;
use cheetah_matches_realtime::grpc::RealtimeInternalService;
use cheetah_matches_realtime::server::manager::ServerManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("matches.relay");
	let (halt_signal, manager) = create_manager();
	let internal_grpc_service = create_internal_grpc_server(manager.clone()).await;
	let admin_grpc_service = create_admin_grpc_server(manager.clone()).await;
	let agones = run_agones_cycle(halt_signal.clone(), manager.clone());
	let (_, _, _) = futures::join!(internal_grpc_service, admin_grpc_service, agones);
	halt_signal.store(true, Ordering::Relaxed);
	Ok(())
}

async fn create_internal_grpc_server(manager: Arc<Mutex<ServerManager>>) -> impl Future<Output = Result<(), tonic::transport::Error>> {
	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
	health_reporter.set_service_status("", ServingStatus::Serving).await;
	let service = cheetah_matches_realtime::grpc::proto::internal::realtime_server::RealtimeServer::new(RealtimeInternalService::new(manager));
	let address = cheetah_libraries_microservice::get_internal_service_binding_addr();
	Server::builder().add_service(service).add_service(health_service).serve(address)
}

async fn create_admin_grpc_server(manager: Arc<Mutex<ServerManager>>) -> impl Future<Output = Result<(), tonic::transport::Error>> {
	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
	health_reporter.set_service_status("", ServingStatus::Serving).await;
	let relay = admin::realtime_server::RealtimeServer::new(RealtimeAdminGRPCService::new(manager.clone()));
	let tracer = admin::command_tracer_server::CommandTracerServer::new(CommandTracerGRPCService::new(manager.clone()));
	let dumper = admin::dump_server::DumpServer::new(DumpGrpcService::new(manager));
	let address = cheetah_libraries_microservice::get_admin_service_binding_addr();
	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(health_service))
		.add_service(tonic_web::enable(dumper))
		.add_service(tonic_web::enable(relay))
		.add_service(tonic_web::enable(tracer))
		.serve(address)
}

fn create_manager() -> (Arc<AtomicBool>, Arc<Mutex<ServerManager>>) {
	let relay_server_binding_address = SocketAddr::from_str("0.0.0.0:5555").unwrap();
	let relay_server_socket = UdpSocket::bind(relay_server_binding_address).unwrap();
	let relay_server = ServerManager::new(relay_server_socket);
	let halt_signal = relay_server.get_halt_signal();
	let server = Arc::new(Mutex::new(relay_server));
	(halt_signal, server)
}
