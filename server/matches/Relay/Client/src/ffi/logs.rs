use widestring::U16CString;

use crate::tracer::TRACER_COLLECTOR;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LogLevel {
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
	collector.set_log_level(log_level);
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
				on_log_message(record.level, string.as_ptr());
			}
		}
	}
}
