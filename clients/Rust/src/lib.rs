#[macro_use]
extern crate lazy_static;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

use log::Level;
use widestring::U16CString;

use cheetah_relay_common::network::hash::HashValue;
use cheetah_relay_common::utils::logger::LogListener;

use crate::client::ffi::Command;
use crate::client::NetworkStatus;
use crate::clients::Clients;

pub mod client;
pub mod clients;

lazy_static! {
    static ref API_REF: Mutex<Clients > = Mutex::new(Default::default());
}

fn execute<F, T>(body: F) -> T
	where F: FnOnce(&mut Clients) -> T
{
	let api = API_REF.lock();
	let api = &mut *(api.unwrap());
	body(api)
}

#[repr(C)]
pub enum LogLevel {
	Info,
	Warn,
	Error,
}

#[no_mangle]
pub extern "C" fn init_logger() {
	LogListener::setup_logger();
	set_max_log_level(LogLevel::Info);
	log::info!("init logger");
}

#[no_mangle]
pub extern "C" fn set_max_log_level(log_level: LogLevel) {
	log::set_max_level(match log_level {
		LogLevel::Info => { log::LevelFilter::Info }
		LogLevel::Warn => { log::LevelFilter::Warn }
		LogLevel::Error => { log::LevelFilter::Error }
	});
}

#[no_mangle]
pub extern "C" fn collect_logs(on_log_message: fn(LogLevel, *const u16)) {
	let collector = &mut cheetah_relay_common::utils::logger::LOG_COLLECTOR.lock().unwrap();
	loop {
		match collector.items.pop_front() {
			None => {
				break;
			}
			Some(record) => {
				let string = U16CString::from_str(record.message).unwrap();
				let level = match record.log_level {
					Level::Error => { LogLevel::Error }
					Level::Warn => { LogLevel::Warn }
					Level::Info => { LogLevel::Info }
					Level::Debug => { LogLevel::Info }
					Level::Trace => { LogLevel::Info }
				};
				on_log_message(level, string.as_ptr());
			}
		}
	}
}


#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn create_client(addr: *const c_char, room_hash: *const c_char, client_hash: *const c_char) -> u16 {
	let server_address = CStr::from_ptr(addr)
		.to_str()
		.unwrap()
		.to_string();
	
	let room_hash = HashValue::from(CStr::from_ptr(room_hash).to_str().unwrap());
	let client_hash = HashValue::from(CStr::from_ptr(client_hash).to_str().unwrap());
	execute(|api| api.create_client(server_address, room_hash, client_hash))
}


#[no_mangle]
pub extern "C" fn get_connection_status(client_id: u16, on_result: fn(NetworkStatus), on_error: fn()) {
	execute(|api| {
		match api.get_connection_status(client_id) {
			Ok(status) => { on_result(status) }
			Err(e) => {
				log::error!("get_connection_status error {:?}", e);
				on_error();
				destroy_client(client_id);
			}
		}
	})
}

#[no_mangle]
pub extern "C" fn receive_commands_from_server(client_id: u16, collector: fn(&Command), on_error: fn()) {
	execute(|api| {
		match api.collect_s2c_commands(client_id, collector) {
			Ok(_) => {}
			Err(e) => {
				log::error!("collect_s2c_commands error {:?}", e);
				on_error();
				destroy_client(client_id);
			}
		}
	});
}


#[no_mangle]
pub extern "C" fn send_command_to_server(client_id: u16, command: &Command, on_error: fn()) {
	execute(|api| {
		log::info!("try command send");
		match api.send_command_to_server(client_id, command) {
			Ok(_) => {
				log::info!("command sended");
			}
			Err(e) => {
				log::error!("send_command_to_server error {:?}", e);
				on_error();
				destroy_client(client_id);
			}
		}
	});
}

#[no_mangle]
pub extern "C" fn destroy_client(client_id: u16) {
	execute(|api| api.destroy_client(client_id));
}




