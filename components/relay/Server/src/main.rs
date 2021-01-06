extern crate stderrlog;

use std::fs;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use clap::{App, Arg};
use log::LevelFilter;
use stderrlog::Timestamp;

use cheetah_relay::room::debug::tracer::CommandTracer;
use cheetah_relay::room::template::config::RoomTemplate;
use cheetah_relay::server::rest::RestServer;
use cheetah_relay::server::Server;

fn main() {
	let (rooms_templates_dir, room_template_path, trace_path, log_level, show_all_trace) = get_cli();
	configure_logger(log_level);
	start_server(rooms_templates_dir, room_template_path, trace_path, show_all_trace);
}

fn get_cli() -> (Option<Vec<String>>, Option<Vec<String>>, Option<String>, Option<String>, bool) {
	const TRACE_ALL_NETWORK_COMMAND: &'static str = "trace-all-network-commands";
	const ROOM_TEMPLATE: &'static str = "room-template";
	const LOG_LEVEL: &'static str = "log-level";
	const COMMAND_TRACE: &'static str = "command-trace";
	const ROOM_TEMPLATES: &'static str = "templates-dir";

	let cli = App::new("Cheetah Relay Server")
		.version("0.0.1")
		.about("Realtime multiplayer game server.")
		.arg(
			Arg::new(ROOM_TEMPLATE)
				.long("room")
				.multiple(true)
				.short('r')
				.required_unless_present(ROOM_TEMPLATES)
				.about("Path to yaml file with config for room template.")
				.takes_value(true),
		)
		.arg(
			Arg::new(ROOM_TEMPLATES)
				.long("templates-dir")
				.multiple(true)
				.short('s')
				.about("Path to directory with yaml files with config for room template.")
				.required_unless_present(ROOM_TEMPLATE)
				.takes_value(true),
		)
		.arg(
			Arg::new(LOG_LEVEL)
				.long(LOG_LEVEL)
				.multiple(false)
				.short('l')
				.required(false)
				.default_value("ERROR")
				.takes_value(true)
				.possible_values(&vec!["TRACE", "DEBUG", "INFO", "WARN", "ERROR"])
				.about("level for log"),
		)
		.arg(
			Arg::new(COMMAND_TRACE)
				.long(COMMAND_TRACE)
				.multiple(false)
				.short('t')
				.required(false)
				.takes_value(true)
				.about("Path to yaml file with config for trace."),
		)
		.arg(
			Arg::new(TRACE_ALL_NETWORK_COMMAND)
				.long(TRACE_ALL_NETWORK_COMMAND)
				.multiple(false)
				.short('a')
				.required(false)
				.takes_value(false)
				.about("Trace all network commands."),
		)
		.get_matches();

	(
		cli.values_of(ROOM_TEMPLATES).map(|v| v.map(|i| i.to_string()).collect()),
		cli.values_of(ROOM_TEMPLATE).map(|v| v.map(|i| i.to_string()).collect()),
		cli.value_of(COMMAND_TRACE).map(|s| s.to_string()),
		cli.value_of(LOG_LEVEL).map(|s| s.to_string()),
		cli.is_present(TRACE_ALL_NETWORK_COMMAND),
	)
}

fn configure_logger(log_level: Option<String>) {
	let level = match log_level {
		None => LevelFilter::Error,
		Some(log_level) => match log_level.as_str() {
			"TRACE" => LevelFilter::Trace,
			"DEBUG" => LevelFilter::Debug,
			"INFO" => LevelFilter::Info,
			"WARN" => LevelFilter::Warn,
			"ERROR" => LevelFilter::Error,
			_ => LevelFilter::Error,
		},
	};
	init_logger(level);
}

fn start_server(
	rooms_templates_dir: Option<Vec<String>>,
	room_templates_path: Option<Vec<String>>,
	trace_path: Option<String>,
	show_all_trace: bool,
) {
	let socket = UdpSocket::bind(SocketAddr::from_str("0.0.0.0:5000").unwrap()).unwrap();
	let tracer = if show_all_trace {
		CommandTracer::new_with_allow_all()
	} else {
		trace_path
			.map(|path| CommandTracer::load_from_file(path).unwrap())
			.unwrap_or(CommandTracer::new_with_deny_all())
	};

	let mut server = Server::new(socket, tracer);

	match room_templates_path {
		None => {}
		Some(room_templates_path) => {
			room_templates_path.iter().for_each(|path| {
				let room_template = RoomTemplate::load_from_file(path).unwrap();
				server.register_room(room_template).ok().unwrap();
			});
		}
	}

	match rooms_templates_dir {
		None => {}
		Some(rooms_templates_dir) => {
			rooms_templates_dir.iter().for_each(|directory| {
				println!("directory {:?}", directory);
				let map = fs::read_dir(directory).unwrap();
				let map = map.map(|e| e.unwrap().path());
				map.for_each(|path| {
					let room_template = RoomTemplate::load_from_file(path.as_path().to_str().unwrap()).unwrap();
					server.register_room(room_template).ok().unwrap();
				});
			});
		}
	}

	let halt_signal = server.get_halt_signal().clone();
	let server = Arc::new(Mutex::new(server));
	RestServer::run(server.clone()).join().unwrap().unwrap();
	halt_signal.store(true, Ordering::Relaxed);
}

fn init_logger(verbosity: LevelFilter) {
	stderrlog::new()
		.verbosity(verbosity as usize)
		.show_level(true)
		.timestamp(Timestamp::Second)
		.init()
		.unwrap();
}
