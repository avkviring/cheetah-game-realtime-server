extern crate stderrlog;

use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use stderrlog::Timestamp;

use cheetah_relay::server::Server;
use cheetah_relay::server::rest::DumpRestServer;

fn main() {
	init_logger();
	start_server();
}

fn start_server() {
	let socket = UdpSocket::bind(SocketAddr::from_str("0.0.0.0:5000").unwrap()).unwrap();
	let server = Server::new(socket, true);
	let halt_signal = server.get_halt_signal().clone();
	let server = Arc::new(Mutex::new(server));
	DumpRestServer::run(server.clone()).join().unwrap().unwrap();
	halt_signal.store(true, Ordering::Relaxed);
}


fn init_logger() {
	stderrlog::new()
		.verbosity(2)
		.show_level(true)
		.timestamp(Timestamp::Second)
		.init()
		.unwrap();
}