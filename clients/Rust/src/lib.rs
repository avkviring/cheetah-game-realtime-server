#[macro_use]
extern crate lazy_static;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Mutex;

use cheetah_relay_common::network::hash::HashValue;

use crate::client::NetworkStatus;
use crate::client::ffi::CommandFFI;
use crate::clients::Clients;
use crate::log::Logger;

pub mod client;
pub mod clients;
pub mod log;

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
pub extern "C" fn get_connection_status(client: u16) -> NetworkStatus {
	execute(|api| api.get_connection_status(client).ok().unwrap())
}

#[no_mangle]
pub extern "C" fn receive_commands_from_server<F>(client_id: u16, collector: F) where F: FnMut(&CommandFFI)->()  {
	execute(|api| api.collect_s2c_commands(client_id, collector));
}

#[no_mangle]
pub extern "C" fn send_command_to_server(client_id: u16, command: &CommandFFI) {
	execute(|api| api.send_command_to_server(client_id, command));
}

#[no_mangle]
pub extern "C" fn destroy_client(client_id: u16) {
	execute(|api| api.destroy_client(client_id));
}

#[no_mangle]
pub extern "C" fn collect_logs(collector: fn(*const c_char)) {
	Logger::collect_logs(collector);
}




