use widestring::U16CString;

use cheetah_protocol::trace_collector::TRACER_COLLECTOR;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LogLevel {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
}

#[no_mangle]
pub extern "C" fn init_logger() {
	set_max_log_level(LogLevel::Error);
}

#[no_mangle]
pub extern "C" fn set_max_log_level(log_level: LogLevel) {
	let collector = &mut TRACER_COLLECTOR.lock().unwrap();
	collector.set_log_level(match log_level {
		LogLevel::Info => tracing_core::Level::INFO,
		LogLevel::Warn => tracing_core::Level::WARN,
		LogLevel::Error => tracing_core::Level::ERROR,
		LogLevel::Trace => tracing_core::Level::TRACE,
		LogLevel::Debug => tracing_core::Level::DEBUG,
	});
}

#[no_mangle]
pub extern "C" fn collect_logs(on_log_message: extern "C" fn(LogLevel, *const u16)) {
	let collector = &mut TRACER_COLLECTOR.lock().unwrap();
	loop {
		match collector.items.pop_front() {
			None => {
				break;
			}
			Some(record) => {
				let string = U16CString::from_str(record.message).unwrap();

				let level = match record.level {
					tracing_core::Level::ERROR => LogLevel::Error,
					tracing_core::Level::INFO => LogLevel::Info,
					tracing_core::Level::WARN => LogLevel::Warn,
					tracing_core::Level::DEBUG => LogLevel::Debug,
					tracing_core::Level::TRACE => LogLevel::Trace,
				};
				on_log_message(level, string.as_ptr());
			}
		}
	}
}
