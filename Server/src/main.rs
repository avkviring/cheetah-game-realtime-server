extern crate stderrlog;


use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use stderrlog::Timestamp;

use cheetah_relay::server::ServerBuilder;

fn main() {
	init_logger();
	//init_rest();
	start_server();
}

fn start_server() {
	let server = ServerBuilder::new("127.0.0.1:5000".to_string()).enable_auto_create_room_and_client().build();
	
	let running = Arc::new(AtomicBool::new(true));
	let r = running.clone();
	
	ctrlc::set_handler(move || {
		r.store(false, Ordering::SeqCst);
	}).expect("Error setting Ctrl-C handler");
	
	while running.load(Ordering::SeqCst) {
		thread::sleep(Duration::from_secs(1));
	}
	drop(server); // для наглядности
}

fn init_logger() {
	stderrlog::new()
		.verbosity(4)
		.show_level(true)
		.timestamp(Timestamp::Second)
		.init()
		.unwrap();
}