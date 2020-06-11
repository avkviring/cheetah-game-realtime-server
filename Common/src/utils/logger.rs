use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Debug)]
pub struct LogCollector {
	pub items: VecDeque<String>
}

#[derive(Debug)]
struct LogRecord {}

lazy_static! {
	pub static ref LOG_COLLECTOR:Mutex<LogCollector> = Mutex::new(LogCollector::new());
}

impl LogCollector {
	fn new() -> Self {
		LogCollector {
			items: Default::default()
		}
	}
}

pub struct LogListener;

static LOG_LISTENER: LogListener = LogListener {};

impl log::Log for LogListener {
	fn enabled(&self, metadata: &log::Metadata) -> bool {
		true
	}
	
	fn log(&self, record: &log::Record) {
		let mut collector = LOG_COLLECTOR.lock().unwrap();
		let message = format!("[{}] ({} in {}) {}", record.level(), record.file().unwrap(), record.line().unwrap(), record.args());
		println!("{}", message);
		collector.items.push_back(message);
	}
	
	fn flush(&self) {}
}

impl LogListener {
	pub fn setup_logger() {
		log::set_logger(&LOG_LISTENER).unwrap();
		log::set_max_level(log::LevelFilter::Trace);
	}
}
