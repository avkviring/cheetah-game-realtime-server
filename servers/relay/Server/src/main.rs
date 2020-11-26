extern crate stderrlog;

use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::atomic::Ordering;

use stderrlog::Timestamp;

use cheetah_relay::server::Server;

fn main() {
	init_logger();
	start_server();
}

fn start_server() {
	let socket = UdpSocket::bind(SocketAddr::from_str("0.0.0.0:5000").unwrap()).unwrap();
	let server = Server::new(socket, true);
	let halt_signal = server.get_halt_signal().clone();
	ctrlc::set_handler(move || {
		halt_signal.store(true, Ordering::Relaxed);
	}).expect("Error setting Ctrl-C handler");
	server.join();
}


fn init_logger() {
	stderrlog::new()
		.verbosity(6)
		.show_level(true)
		.timestamp(Timestamp::Second)
		.init()
		.unwrap();
}