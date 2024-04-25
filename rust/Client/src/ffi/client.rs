use cheetah_game_realtime_protocol::disconnect::command::DisconnectByCommandReason;
use cheetah_game_realtime_protocol::frame::disconnected_reason::DisconnectedReason;
use cheetah_game_realtime_protocol::frame::member_private_key::MemberPrivateKey;
use cheetah_game_realtime_protocol::{RoomId, RoomMemberId};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::clients::registry::ClientId;
use crate::ffi::command::{BufferFFI, S2CCommandFFI};
use crate::ffi::{execute, execute_with_client, ClientError, LAST_ERROR};
use cheetah_common::network::ConnectionStatus;
use cheetah_common::room::buffer::Buffer;

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
	RetransmitOverflow,
	DisconnectedByTimeout,
	DisconnectedByClientStopped,
	DisconnectedByRoomDeleted,
	DisconnectedByMemberDeleted,
}

#[no_mangle]
pub extern "C" fn get_connection_status(client_id: ClientId, result: &mut ConnectionStatusFFI) -> u8 {
	execute_with_client(client_id, |client| {
		let status = client.get_connection_status().map_err(|e| ClientError::ConnectionStatusMutexError(format!("{e:?}")));
		match status {
			Ok(status) => {
				let ffi_status = match status {
					ConnectionStatus::Connecting => ConnectionStatusFFI::Connecting,
					ConnectionStatus::Connected => ConnectionStatusFFI::Connected,
					ConnectionStatus::Disconnected(disconnect_reason) => match disconnect_reason {
						DisconnectedReason::IOError(_) => ConnectionStatusFFI::DisconnectedByIOError,
						DisconnectedReason::Timeout => ConnectionStatusFFI::DisconnectedByTimeout,
						DisconnectedReason::Command(reason) => match reason {
							DisconnectByCommandReason::ClientStopped => ConnectionStatusFFI::DisconnectedByClientStopped,
							DisconnectByCommandReason::RoomDeleted => ConnectionStatusFFI::DisconnectedByRoomDeleted,
							DisconnectByCommandReason::MemberDeleted => ConnectionStatusFFI::DisconnectedByMemberDeleted,
						},
						DisconnectedReason::RetransmitOverflow => ConnectionStatusFFI::RetransmitOverflow,
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
pub extern "C" fn destroy_client_without_disconnect(client: ClientId) -> u8 {
	execute(|registry| match registry.destroy_client_without_disconnect(client) {
		None => Err(ClientError::ClientNotFound(client)),
		Some(_) => Ok(()),
	})
}

#[no_mangle]
pub extern "C" fn receive(client_id: ClientId, out_commands: *mut S2CCommandFFI, count: &mut u16) -> u8 {
	execute_with_client(client_id, |client| unsafe {
		client.receive(out_commands, count);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn get_server_time(client_id: ClientId, server_out_time: &mut u64) -> u8 {
	execute_with_client(client_id, |client| {
		*server_out_time = client.get_server_time().unwrap_or_default();
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn set_rtt_emulation(client_id: ClientId, rtt_in_ms: u64, rtt_dispersion: f64) -> u8 {
	execute_with_client(client_id, |client| Ok(client.set_rtt_emulation(Duration::from_millis(rtt_in_ms), rtt_dispersion)?))
}

#[no_mangle]
pub extern "C" fn set_drop_emulation(client_id: ClientId, drop_probability: f64, drop_time_in_ms: u64) -> u8 {
	execute_with_client(client_id, |client| Ok(client.set_drop_emulation(drop_probability, Duration::from_millis(drop_time_in_ms))?))
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
	buffer.len = length as u16;
	buffer.buffer[0..length].copy_from_slice(msg);
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Statistics {
	pub last_frame_id: u64,
	pub rtt_in_ms: u64,
	pub recv_packet_count: u64,
	pub send_packet_count: u64,
	pub recv_size: u64,
	pub send_size: u64,
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn create_client(
	connection_id: u64,
	addr: *const c_char,
	member_id: RoomMemberId,
	room_id: RoomId,
	private_key_buffer: &BufferFFI,
	disconnect_time_in_sec: u64,
	out_client_id: &mut u16,
) -> u8 {
	let server_address = CStr::from_ptr(addr).to_str().unwrap();
	let mut private_key = [0; 32];
	private_key.copy_from_slice(&private_key_buffer.buffer[0..32]);
	do_create_client(connection_id, server_address, member_id, room_id, &private_key.as_slice().into(), disconnect_time_in_sec, out_client_id)
}

pub fn do_create_client(
	connection_id: u64,
	server_address: &str,
	member_id: RoomMemberId,
	room_id: RoomId,
	private_key: &MemberPrivateKey,
	disconnect_timeout_in_sec: u64,
	out_client_id: &mut u16,
) -> u8 {
	execute(|api| {
		api.create_client(connection_id, server_address, member_id, room_id, private_key.clone(), disconnect_timeout_in_sec)
			.map(|client_id| {
				*out_client_id = client_id;
				Ok(())
			})?
	})
}
