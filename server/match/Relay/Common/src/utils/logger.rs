use lazy_static::lazy_static;
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Debug)]
pub struct LogCollector {
	pub items: VecDeque<LogRecord>,
}

#[derive(Debug)]
pub struct LogRecord {
	pub log_level: log::Level,
	pub message: String,
}

lazy_static! {
	pub static ref LOG_COLLECTOR: Mutex<LogCollector> = Mutex::new(LogCollector::new());
}

impl LogCollector {
	fn new() -> Self {
		LogCollector { items: Default::default() }
	}
}

pub struct LogListener;

static LOG_LISTENER: LogListener = LogListener {};

impl log::Log for LogListener {
	fn enabled(&self, _metadata: &log::Metadata) -> bool {
		true
	}

	fn log(&self, record: &log::Record) {
		let mut collector = LOG_COLLECTOR.lock().unwrap();
		let message = match record.level() {
			log::Level::Trace => {
				format!("{}", record.args())
			}
			log::Level::Info => {
				format!("{}", record.args())
			}
			_ => {
				format!("({} in {}) {}", record.file().unwrap(), record.line().unwrap(), record.args())
			}
		};
		println!("[{:?}] {}", record.level(), message);
		collector.items.push_back(LogRecord {
			log_level: record.level(),
			message,
		});
	}

	fn flush(&self) {}
}

impl LogListener {
	pub fn setup_logger() {
		match log::set_logger(&LOG_LISTENER) {
			Ok(_) => {
				log::set_max_level(log::LevelFilter::Trace);
			}
			Err(_) => {}
		}
	}
}
