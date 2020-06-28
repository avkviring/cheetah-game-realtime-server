extern crate stderrlog;

use std::thread;
use std::time::Duration;

use stderrlog::Timestamp;

use cheetah_relay::server::{Server, ServerBuilder};

fn main() {
	init_logger();
	//init_rest();
	start_server();
}

fn start_server() {
	let server = ServerBuilder::new("127.0.0.1:5000".to_string()).enable_auto_create_room_and_client().build();
	loop {
		thread::sleep(Duration::from_secs(1));
	}
}

fn init_logger() {
	stderrlog::new()
		.verbosity(4)
		.show_level(true)
		.timestamp(Timestamp::Second)
		.init()
		.unwrap();
}