use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::atomic::Ordering;
use std::time::Duration;

use cheetah_matches_realtime_common::network::client::{ConnectionStatus, DisconnectedReason};
use cheetah_matches_realtime_common::room::{MemberPrivateKey, RoomId, RoomMemberId};

use crate::clients::registry::ClientId;
use crate::ffi::{execute, execute_with_client, BufferFFI, ClientError, LAST_ERROR};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum ConnectionStatusFFI {
	Connecting,
	///
	/// Соединение установлено
	///
	Connected,
	///
	/// Соединение закрыто
	///
	DisconnectedByIOError,
	DisconnectedByRetryLimit,
	DisconnectedByTimeout,
	DisconnectedByCommand,
}

#[no_mangle]
pub extern "C" fn get_connection_status(client_id: ClientId, result: &mut ConnectionStatusFFI) -> u8 {
	execute_with_client(client_id, |client| {
		let status = client
			.get_connection_status()
			.map_err(|e| ClientError::ConnectionStatusMutexError(format!("{:?}", e)));
		match status {
			Ok(status) => {
				let ffi_status = match status {
					ConnectionStatus::Connecting => ConnectionStatusFFI::Connecting,
					ConnectionStatus::Connected => ConnectionStatusFFI::Connected,
					ConnectionStatus::Disconnected(disconnect_reason) => match disconnect_reason {
						DisconnectedReason::IOError(_) => ConnectionStatusFFI::DisconnectedByIOError,
						DisconnectedReason::ByRetryLimit => ConnectionStatusFFI::DisconnectedByRetryLimit,
						DisconnectedReason::ByTimeout => ConnectionStatusFFI::DisconnectedByTimeout,
						DisconnectedReason::ByCommand => ConnectionStatusFFI::DisconnectedByCommand,
					},
				};
				*result = ffi_status;

				Ok(())
			}
			Err(e) => Err(e),
		}
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
	execute_with_client(client_id, |client| {
		client.receive();
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn get_server_time(client_id: ClientId, server_out_time: &mut u64) -> u8 {
	execute_with_client(client_id, |client| {
		*server_out_time = client.get_server_time().unwrap_or(0_u64);
		Ok(())
	})
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
		let shared_statistics = &client.shared_statistics;
		statistics.last_frame_id = shared_statistics.current_frame_id.load(Ordering::Relaxed);
		statistics.rtt_in_ms = shared_statistics.rtt_in_ms.load(Ordering::Relaxed);
		statistics.average_retransmit_frames = shared_statistics.average_retransmit_frames.load(Ordering::Relaxed);
		statistics.recv_packet_count = shared_statistics.recv_packet_count.load(Ordering::Relaxed);
		statistics.send_packet_count = shared_statistics.send_packet_count.load(Ordering::Relaxed);
		statistics.recv_size = shared_statistics.recv_size.load(Ordering::Relaxed);
		statistics.send_size = shared_statistics.send_size.load(Ordering::Relaxed);
		Ok(())
	})
}

#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
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
	pub recv_packet_count: u64,
	pub send_packet_count: u64,
	pub recv_size: u64,
	pub send_size: u64,
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn create_client(
	addr: *const c_char,
	member_id: RoomMemberId,
	room_id: RoomId,
	user_private_key_buffer: &BufferFFI,
	start_frame_id: u64,
	out_client_id: &mut u16,
) -> u8 {
	let server_address = CStr::from_ptr(addr).to_str().unwrap();
	let mut user_private_key = [0; 32];
	user_private_key.copy_from_slice(&user_private_key_buffer.buffer[0..32]);
	do_create_client(
		server_address,
		member_id,
		room_id,
		&user_private_key.as_slice().into(),
		start_frame_id,
		out_client_id,
	)
}

pub fn do_create_client(
	server_address: &str,
	member_id: RoomMemberId,
	room_id: RoomId,
	user_private_key: &MemberPrivateKey,
	start_frame_id: u64,
	out_client_id: &mut u16,
) -> u8 {
	execute(
		|api| match api.create_client(server_address, member_id, room_id, user_private_key.clone(), start_frame_id) {
			Ok(client_id) => {
				*out_client_id = client_id;
				Ok(())
			}
			Err(e) => Err(ClientError::CreateClientError(format!("{:?}", e))),
		},
	)
}
