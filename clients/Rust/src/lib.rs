#[macro_use]
extern crate lazy_static;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::process::exit;
use std::sync::Mutex;

use crate::client::command::upload::UploadObjectC2S;
use crate::client::ffi::S2CCommandFFI;
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
pub unsafe extern "C" fn create_client(addr: *const c_char) -> u16 {
	let server_address = unsafe { CStr::from_ptr(addr) }
		.to_str()
		.unwrap()
		.to_string();
	
	execute(|api| api.create_client(server_address))
}

#[no_mangle]
pub extern "C" fn receive_commands_from_server(client_id: u16, collector: fn(&S2CCommandFFI)) {
	execute(|api| api.collect_s2c_commands(client_id, collector));
}

#[no_mangle]
pub extern "C" fn destroy_client(client_id: u16) {
	execute(|api| api.destroy_client(client_id));
}

#[no_mangle]
pub extern "C" fn collect_logs(collector: fn(*const c_char)) {
	Logger::collect_logs(collector);
}


#[no_mangle]
pub extern "C" fn send_command_upload_object(client_id: u16, command: &S2CCommandFFI) {
	execute(|api| api.send_command_to_server(client_id, command))
}

