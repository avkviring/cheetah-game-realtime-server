use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::atomic::Ordering;
use std::time::Duration;

use cheetah_matches_relay_common::network::client::ConnectionStatus;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId, UserPrivateKey};

use crate::clients::registry::ClientId;
use crate::ffi::{execute, execute_with_client, BufferFFI, ClientError, LAST_ERROR};

#[no_mangle]
pub extern "C" fn get_connection_status(client_id: ClientId, result: &mut ConnectionStatus) -> u8 {
	execute_with_client(client_id, |client| {
		client
			.get_connection_status(result)
			.map_err(|e| ClientError::ConnectionStatusMutexError(format!("{:?}", e)))
	})
}

#[no_mangle]
pub extern "C" fn destroy_client(client: ClientId) -> u8 {
	execute(|registry| match registry.destroy_client(client) {
		None => Err(ClientError::ClientNotFound(client)),
		Some(_) => Ok(()),
	})
}

#[no_mangle]
pub extern "C" fn receive(client_id: ClientId) -> u8 {
	execute_with_client(client_id, |client| Ok(client.receive()))
}

#[no_mangle]
pub extern "C" fn set_rtt_emulation(client_id: ClientId, rtt_in_ms: u64, rtt_dispersion: f64) -> u8 {
	execute_with_client(client_id, |client| {
		Ok(client.set_rtt_emulation(Duration::from_millis(rtt_in_ms), rtt_dispersion)?)
	})
}

#[no_mangle]
pub extern "C" fn set_drop_emulation(client_id: ClientId, drop_probability: f64, drop_time_in_ms: u64) -> u8 {
	execute_with_client(client_id, |client| {
		Ok(client.set_drop_emulation(drop_probability, Duration::from_millis(drop_time_in_ms))?)
	})
}

#[no_mangle]
pub extern "C" fn reset_emulation(client_id: ClientId) -> u8 {
	execute_with_client(client_id, |client| Ok(client.reset_emulation()?))
}

#[no_mangle]
pub extern "C" fn get_statistics(client_id: ClientId, statistics: &mut Statistics) -> u8 {
	execute_with_client(client_id, |client| {
		statistics.last_frame_id = client.current_frame_id.load(Ordering::Relaxed);
		statistics.rtt_in_ms = client.rtt_in_ms.load(Ordering::Relaxed);
		statistics.average_retransmit_frames = client.average_retransmit_frames.load(Ordering::Relaxed);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn get_last_error_msg(buffer: &mut BufferFFI) {
	let msg = LAST_ERROR.lock().unwrap();
	let msg = msg.as_bytes();
	let length = msg.len();
	buffer.len = length as u8;
	buffer.buffer[0..length].copy_from_slice(msg);
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Statistics {
	pub last_frame_id: u64,
	pub rtt_in_ms: u64,
	pub average_retransmit_frames: u32,
}

#[no_mangle]
pub unsafe extern "C" fn create_client(
	addr: *const c_char,
	member_id: RoomMemberId,
	room_id: RoomId,
	user_private_key_buffer: &BufferFFI,
	start_frame_id: u64,
	out_client_id: &mut u16,
) -> u8 {
	let server_address = CStr::from_ptr(addr).to_str().unwrap().to_string();
	let mut user_private_key = [0; 32];
	user_private_key.copy_from_slice(&user_private_key_buffer.buffer[0..32]);
	do_create_client(
		server_address,
		member_id,
		room_id,
		&user_private_key,
		start_frame_id,
		out_client_id,
	)
}

pub fn do_create_client(
	server_address: String,
	member_id: RoomMemberId,
	room_id: RoomId,
	user_private_key: &UserPrivateKey,
	start_frame_id: u64,
	out_client_id: &mut u16,
) -> u8 {
	execute(
		|api| match api.create_client(server_address, member_id, room_id, *user_private_key, start_frame_id) {
			Ok(client_id) => {
				*out_client_id = client_id;
				Ok(())
			}
			Err(e) => Err(ClientError::CreateClientError(format!("{:?}", e))),
		},
	)
}
