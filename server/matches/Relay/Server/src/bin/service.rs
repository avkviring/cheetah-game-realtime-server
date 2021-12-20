extern crate stderrlog;

use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use futures::Future;
use tonic::transport::Server;

use cheetah_matches_relay::agones::run_agones_cycle;
use cheetah_matches_relay::debug::dump::DumpGrpcService;
use cheetah_matches_relay::debug::grpc::RelayAdminGRPCService;
use cheetah_matches_relay::debug::proto::admin;
use cheetah_matches_relay::debug::tracer::grpc::CommandTracerGRPCService;
use cheetah_matches_relay::factory::RelayGRPCService;
use cheetah_matches_relay::server::manager::RelayManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("relay");
	let (halt_signal, manager) = create_manager();
	let internal_grpc_service = create_internal_grpc_server(manager.clone());
	let admin_grpc_service = create_admin_grpc_server(manager.clone());
	let agones = run_agones_cycle(halt_signal.clone(), manager.clone());
	let (_, _, _) = futures::join!(internal_grpc_service, admin_grpc_service, agones);
	halt_signal.store(true, Ordering::Relaxed);
	Result::Ok(())
}

fn create_internal_grpc_server(manager: Arc<Mutex<RelayManager>>) -> impl Future<Output = Result<(), tonic::transport::Error>> {
	let service = cheetah_matches_relay::factory::proto::internal::relay_server::RelayServer::new(RelayGRPCService::new(manager));
	let address = cheetah_microservice::get_internal_service_binding_addr();
	Server::builder().add_service(service).serve(address)
}

fn create_admin_grpc_server(manager: Arc<Mutex<RelayManager>>) -> impl Future<Output = Result<(), tonic::transport::Error>> {
	let relay = admin::relay_server::RelayServer::new(RelayAdminGRPCService::new(manager.clone()));
	let tracer = admin::command_tracer_server::CommandTracerServer::new(CommandTracerGRPCService::new(manager.clone()));
	let dumper = admin::dump_server::DumpServer::new(DumpGrpcService::new(manager));
	let address = cheetah_microservice::get_admin_service_binding_addr();
	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(dumper))
		.add_service(tonic_web::enable(relay))
		.add_service(tonic_web::enable(tracer))
		.serve(address)
}

fn create_manager() -> (Arc<AtomicBool>, Arc<Mutex<RelayManager>>) {
	let relay_server_binding_address = SocketAddr::from_str("0.0.0.0:5555").unwrap();
	let relay_server_socket = UdpSocket::bind(relay_server_binding_address).unwrap();
	let relay_server = RelayManager::new(relay_server_socket);
	let halt_signal = relay_server.get_halt_signal();
	let server = Arc::new(Mutex::new(relay_server));
	(halt_signal, server)
}
