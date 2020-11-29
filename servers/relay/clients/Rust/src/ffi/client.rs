use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::atomic::Ordering;

use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};
use cheetah_relay_common::udp::client::ConnectionStatus;

use crate::ffi::{execute, execute_with_client, BufferFFI};
use crate::registry::ClientId;

#[no_mangle]
pub extern "C" fn get_connection_status(result: &mut ConnectionStatus) -> bool {
	match execute_with_client(|api| api.get_connection_status()) {
		Ok(status) => {
			*result = status;
			true
		}
		Err(_) => false,
	}
}

#[no_mangle]
pub extern "C" fn set_current_client(client_id: ClientId) -> bool {
	execute(|api| match api.controllers.get(&client_id) {
		None => false,
		Some(_) => {
			api.current_client = Some(client_id);
			true
		}
	})
}

#[no_mangle]
pub extern "C" fn destroy_client() -> bool {
	execute(|api| api.destroy_client())
}

#[no_mangle]
pub extern "C" fn receive() -> bool {
	execute_with_client(|client| client.receive()).is_ok()
}

#[no_mangle]
pub extern "C" fn get_frame_id(out_frame_id: &mut u64) -> bool {
	execute_with_client(|client| {
		*out_frame_id = client.current_frame_id.load(Ordering::Relaxed);
	})
	.is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn create_client(
	addr: *const c_char,
	user_public_key: UserPublicKey,
	user_private_key_buffer: &BufferFFI,
	start_frame_id: u64,
	out_client_id: &mut u16,
) -> bool {
	let server_address = CStr::from_ptr(addr).to_str().unwrap().to_string();
	let mut user_private_key = [0; 32];
	user_private_key.copy_from_slice(&user_private_key_buffer.buffer[0..32]);
	do_create_client(server_address, user_public_key, &user_private_key, start_frame_id, out_client_id)
}

pub fn do_create_client(
	server_address: String,
	user_public_key: UserPublicKey,
	user_private_key: &UserPrivateKey,
	start_frame_id: u64,
	out_client_id: &mut u16,
) -> bool {
	execute(
		|api| match api.create_client(server_address, user_public_key, user_private_key.clone(), start_frame_id) {
			Ok(client_id) => {
				*out_client_id = client_id;
				true
			}
			Err(_) => false,
		},
	)
}
