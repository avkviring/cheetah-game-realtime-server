use std::ffi::CStr;
use std::os::raw::c_char;

use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};
use cheetah_relay_common::udp::client::ConnectionStatus;

use crate::registry::ClientId;
use crate::ffi::{execute, execute_with_client};

#[no_mangle]
pub extern "C" fn get_connection_status(on_result: extern fn(ConnectionStatus), on_error: extern fn()) {
	match execute_with_client(|api| { api.get_connection_status() }) {
		Ok(status) => {
			on_result(status)
		}
		Err(_) => {
			on_error()
		}
	}
}

#[no_mangle]
pub extern "C" fn set_current_client(client_id: ClientId) -> bool {
	execute(|api| {
		match api.controllers.get(&client_id) {
			None => {
				false
			}
			Some(_) => {
				api.current_client = Some(client_id);
				true
			}
		}
	})
}

#[no_mangle]
pub extern "C" fn destroy_client(client_id: u16) {
	execute(|api| api.destroy_client(client_id));
}

#[no_mangle]
pub unsafe extern "C" fn create_client(
	addr: *const c_char,
	user_public_key: UserPublicKey,
	user_private_key: &UserPrivateKey,
	on_error: extern fn(),
	on_create: extern fn(u16),
) {
	let server_address = CStr::from_ptr(addr)
		.to_str()
		.unwrap()
		.to_string();
	do_create_client(server_address, user_public_key, user_private_key, || on_error(), |c| on_create(c));
}

pub fn do_create_client<E, C>(
	server_address: String,
	user_public_key: UserPublicKey,
	user_private_key: &UserPrivateKey,
	on_error: E,
	on_create: C,
) where E: FnOnce() -> (), C: FnOnce(u16) -> () {
	execute(|api| {
		match api.create_client(server_address, user_public_key, user_private_key.clone()) {
			Ok(client_id) => { on_create(client_id) }
			Err(_) => { on_error() }
		}
	});
}

