extern crate stderrlog;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use stderrlog::Timestamp;

use cheetah_relay::server::Server;

fn main() {
	init_logger();
	//init_rest();
	start_server();
}

fn start_server() {
	let running_flag = Arc::new(AtomicBool::new(true));
	let server = Server::new(SocketAddr::from_str("0.0.0.0:5000").unwrap(), running_flag.clone());
	let r = running_flag.clone();
	
	
	ctrlc::set_handler(move || {
		r.store(false, Ordering::Relaxed);
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