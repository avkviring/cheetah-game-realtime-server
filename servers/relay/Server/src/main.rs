extern crate stderrlog;

use std::io::Read;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use clap::{App, Arg, Clap};
use stderrlog::Timestamp;

use cheetah_relay::server::rest::DumpRestServer;
use cheetah_relay::server::Server;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
struct Opts {
	#[clap(short, long)]
	rooms: Vec<String>,
}

fn main() {
	let matches = App::new("Cheetah Relay Server")
		.version("0.0.1")
		.about("Realtime multiplayer game server.")
		.arg(
			Arg::new("room-template")
				.long("room")
				.multiple(true)
				.short('r')
				.required(true)
				.about("Config for rooms template (see Server/examples/)")
				.takes_value(true),
		)
		.get_matches();

	match matches.values_of("room-template") {
		None => {}
		Some(room_templates_path) => {
			init_logger();
			start_server(room_templates_path);
		}
	}
}

fn start_server(room_templates_path: clap::Values) {
	let socket = UdpSocket::bind(SocketAddr::from_str("0.0.0.0:5000").unwrap()).unwrap();
	let mut server = Server::new(socket);

	room_templates_path.for_each(|path| {
		let mut file = std::fs::File::open(path).unwrap();
		let mut content = String::default();
		file.read_to_string(&mut content).unwrap();
		let room_template = serde_yaml::from_str(content.as_str()).unwrap();
		server.register_room(room_template).ok().unwrap();
	});

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
