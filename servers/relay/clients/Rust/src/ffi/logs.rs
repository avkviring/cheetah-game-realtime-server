use log::Level;
use widestring::U16CString;

use cheetah_relay_common::utils::logger::LogListener;

#[repr(C)]
pub enum LogLevel {
	Info,
	Warn,
	Error,
}

#[no_mangle]
pub extern "C" fn init_logger() {
	LogListener::setup_logger();
	set_max_log_level(LogLevel::Error);
}

#[no_mangle]
pub extern "C" fn set_max_log_level(log_level: LogLevel) {
	log::set_max_level(match log_level {
		LogLevel::Info => log::LevelFilter::Info,
		LogLevel::Warn => log::LevelFilter::Warn,
		LogLevel::Error => log::LevelFilter::Error,
	});
}

#[no_mangle]
pub extern "C" fn collect_logs(on_log_message: extern "C" fn(LogLevel, *const u16)) {
	let collector = &mut cheetah_relay_common::utils::logger::LOG_COLLECTOR.lock().unwrap();
	loop {
		match collector.items.pop_front() {
			None => {
				break;
			}
			Some(record) => {
				let string = U16CString::from_str(record.message).unwrap();
				let level = match record.log_level {
					Level::Error => LogLevel::Error,
					Level::Warn => LogLevel::Warn,
					Level::Info => LogLevel::Info,
					Level::Debug => LogLevel::Info,
					Level::Trace => LogLevel::Info,
				};
				on_log_message(level, string.as_ptr());
			}
		}
	}
}
