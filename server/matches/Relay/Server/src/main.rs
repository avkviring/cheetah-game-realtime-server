extern crate stderrlog;

use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use futures::Future;
use tonic::transport::Server;

use cheetah_matches_relay::agones::run_agones_cycle;
use cheetah_matches_relay::grpc::RelayGRPCService;
use cheetah_matches_relay::server::RelayServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("relay");

	let (halt_signal, relay_server) = create_relay_server();
	let grpc_await = create_grpc_server(relay_server.clone());
	let agones = run_agones_cycle(halt_signal.clone(), relay_server.clone());
	futures::join!(grpc_await, agones);
	halt_signal.store(true, Ordering::Relaxed);
	Result::Ok(())
}

fn create_grpc_server(relay_server: Arc<Mutex<RelayServer>>) -> impl Future<Output = Result<(), tonic::transport::Error>> {
	let service = cheetah_matches_relay::proto::internal::relay_server::RelayServer::new(RelayGRPCService::new(relay_server));
	let address = cheetah_microservice::get_internal_service_binding_addr();
	Server::builder().add_service(service).serve(address)
}

fn create_relay_server() -> (Arc<AtomicBool>, Arc<Mutex<RelayServer>>) {
	let relay_server_binding_address = SocketAddr::from_str("0.0.0.0:5555").unwrap();
	let relay_server_socket = UdpSocket::bind(relay_server_binding_address).unwrap();
	let relay_server = RelayServer::new(relay_server_socket);
	let halt_signal = relay_server.get_halt_signal().clone();
	let server = Arc::new(Mutex::new(relay_server));
	(halt_signal, server)
}
