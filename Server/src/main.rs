extern crate stderrlog;

use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};


use stderrlog::Timestamp;


use cheetah_relay::server::Server;
use cheetah_relay_common::room::access::AccessGroups;

fn main() {
	init_logger();
	start_server();
}

fn start_server() {
	let halt_signal = Arc::new(AtomicBool::new(false));
	let socket = UdpSocket::bind(SocketAddr::from_str("0.0.0.0:5000").unwrap()).unwrap();
	let mut server = Server::new(socket, halt_signal.clone());
	
	register_test_users(&mut server);
	
	ctrlc::set_handler(move || {
		halt_signal.store(true, Ordering::Relaxed);
	}).expect("Error setting Ctrl-C handler");
	
	server.join();
}


fn register_test_users(server: &mut Server) {
	for room in 0..3 {
		server.register_room(room).ok().unwrap();
		for user in 0..10 {
			let public_key = (room * 255 + user) as u32;
			let mut private_key = [0; 32];
			private_key[0] = room as u8;
			private_key[1] = user as u8;
			server.register_user(room, public_key, private_key, AccessGroups(0b1111)).ok().expect("register user error");
		}
	}
}

fn init_logger() {
	stderrlog::new()
		.verbosity(6)
		.show_level(true)
		.timestamp(Timestamp::Second)
		.init()
		.unwrap();
}