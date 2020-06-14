extern crate stderrlog;

use stderrlog::Timestamp;

use cheetah_relay::server::Server;

fn main() {
	init_logger();
	//init_rest();
	start_server();
}

fn start_server() {
	let _server = Server::new("127.0.0.1:5000".to_string());
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