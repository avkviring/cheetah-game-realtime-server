#[macro_use]
extern crate lazy_static;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Mutex;

use log::Level;


use cheetah_relay_common::room::{RoomId, UserPublicKey};
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
	set_max_log_level(LogLevel::Error);
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
pub extern "C" fn collect_logs(on_log_message: extern fn(LogLevel, *const u16)) {
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
pub unsafe extern "C" fn create_client(addr: *const c_char, room_id: RoomId, user_public_key: UserPublicKey) -> u16 {
	let server_address = CStr::from_ptr(addr)
		.to_str()
		.unwrap()
		.to_string();
	execute(|api| api.create_client(server_address, room_id, user_public_key))
}


pub fn do_get_connection_status<F, E>(client_id: u16, on_result: F, on_error: E) where F: FnOnce(NetworkStatus), E: FnOnce() {
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

pub fn do_receive_commands_from_server<F, E>(client_id: u16, collector: F, on_error: E) where F: FnMut(&Command), E: FnOnce() {
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

pub fn do_send_command_to_server<E>(client_id: u16, command: &Command, on_error: E) where E: FnOnce() {
	execute(|api| {
		match api.send_command_to_server(client_id, command) {
			Ok(_) => {}
			Err(e) => {
				log::error!("send_command_to_server error {:?}", e);
				on_error();
				destroy_client(client_id);
			}
		}
	});
}


#[no_mangle]
pub extern "C" fn receive_commands_from_server(client_id: u16, collector: extern fn(&Command), on_error: extern fn()) {
	do_receive_commands_from_server(client_id, |command: &Command| collector(command), || on_error());
}

#[no_mangle]
pub extern "C" fn send_command_to_server(client_id: u16, command: &Command, on_error: extern fn()) {
	do_send_command_to_server(client_id, command, || on_error())
}

#[no_mangle]
pub extern "C" fn get_connection_status(client_id: u16, on_result: extern fn(NetworkStatus), on_error: extern fn()) {
	do_get_connection_status(client_id, |status| on_result(status), || on_error())
}


#[no_mangle]
pub extern "C" fn destroy_client(client_id: u16) {
	execute(|api| api.destroy_client(client_id));
}




