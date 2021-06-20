extern crate stderrlog;

use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use clap::{App, Arg};
use log::LevelFilter;
use stderrlog::Timestamp;

use cheetah_relay::room::debug::tracer::CommandTracer;
use cheetah_relay::server::rest::RestServer;
use cheetah_relay::server::Server;

fn main() {
	let (trace_path, log_level, show_all_trace, game_port, rest_port) = get_cli();
	configure_logger(log_level);
	start_server(trace_path, show_all_trace, game_port, rest_port);
}

fn get_cli() -> (Option<String>, Option<String>, bool, u16, u16) {
	const TRACE_ALL_NETWORK_COMMAND: &'static str = "trace-all-network-commands";
	const LOG_LEVEL: &'static str = "log-level";
	const COMMAND_TRACE: &'static str = "command-trace";
	const GAME_PORT: &'static str = "game-port";
	const REST_PORT: &'static str = "rest-port";

	let cli = App::new("Cheetah Relay Server")
		.version("1.0.0")
		.about("Realtime multiplayer game server.")
		.author("https://cheetah.games")
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
		.arg(
			Arg::new(GAME_PORT)
				.long(GAME_PORT)
				.multiple(false)
				.required(false)
				.takes_value(false)
				.default_value("5555")
				.about("Listen port for game connections."),
		)
		.arg(
			Arg::new(REST_PORT)
				.long(REST_PORT)
				.multiple(false)
				.required(false)
				.takes_value(false)
				.default_value("8080")
				.about("Listen port rest connections."),
		)
		.get_matches();

	let game_port = cli.value_of(GAME_PORT).unwrap().parse().unwrap();
	let rest_port = cli.value_of(REST_PORT).unwrap().parse().unwrap();
	(
		cli.value_of(COMMAND_TRACE).map(|s| s.to_string()),
		cli.value_of(LOG_LEVEL).map(|s| s.to_string()),
		cli.is_present(TRACE_ALL_NETWORK_COMMAND),
		game_port,
		rest_port,
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

fn start_server(trace_path: Option<String>, show_all_trace: bool, game_port: u16, rest_port: u16) {
	println!("±± cheetah game relay component ±±");
	let socket = UdpSocket::bind(SocketAddr::from_str(format!("0.0.0.0:{}", game_port).as_str()).unwrap())
		.expect("Can not bind port for game server, use --game-port for other port");
	let tracer = if show_all_trace {
		CommandTracer::new_with_allow_all()
	} else {
		trace_path
			.map(|path| CommandTracer::load_from_file(path).unwrap())
			.unwrap_or(CommandTracer::new_with_deny_all())
	};

	let server = Server::new(socket, tracer);
	let halt_signal = server.get_halt_signal().clone();
	let server = Arc::new(Mutex::new(server));
	RestServer::run(server.clone(), rest_port).join().unwrap().unwrap();
	halt_signal.store(true, Ordering::Relaxed);
}

fn init_logger(verbosity: LevelFilter) {
	stderrlog::new()
		.verbosity(verbosity as usize)
		.show_level(true)
		.timestamp(Timestamp::Off)
		.init()
		.unwrap();
}
