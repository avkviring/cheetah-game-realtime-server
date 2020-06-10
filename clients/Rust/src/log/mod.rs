use std::collections::VecDeque;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::Mutex;

pub struct Logger {
	items: VecDeque<String>
}


impl Default for Logger {
	fn default() -> Self {
		Logger {
			items: Default::default()
		}
	}
}

lazy_static! {
    static ref LOG_REF: Mutex<Logger> = Mutex::new(Default::default());
}

impl Logger {
	pub fn collect_logs(collector: fn(*const c_char)) {
		let mut logger = LOG_REF.lock().unwrap();
		loop {
			match logger.items.pop_front() {
				None => {
					break;
				}
				Some(line) => {
					let c_str = CString::new(line).unwrap();
					collector(c_str.as_ptr() as *const c_char);
				}
			}
		}
	}
}