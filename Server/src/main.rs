extern crate stderrlog;


use std::sync::{Arc, Mutex};
use std::thread;
use stderrlog::Timestamp;

use cheetah_relay::network::server::tcp::TCPServer;
use cheetah_relay::rooms::Rooms;




fn main() {
	init_logger();
	//init_rest();
	start_server();
}

fn start_server() {
	let rooms = Arc::new(Mutex::new(Rooms::new()));
	let handler = thread::spawn(|| {
		let mut server = TCPServer::new("0.0.0.0:5050".to_string(), rooms);
		server.start();
	});
	let result = handler.join();
	match result {
		Ok(_) => {}
		Err(e) => {
			log::error!("main: error join to tcp thread {:?}", e);
		}
	}
}

fn init_logger() {
	stderrlog::new()
		.verbosity(4)
		.quiet(false)
		.show_level(true)
		.timestamp(Timestamp::Second)
		.init()
		.unwrap();
}