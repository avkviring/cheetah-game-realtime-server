use std::net::IpAddr;

use crate::ffi::{ServerId, REGISTRY};
use crate::EmbeddedServerWrapper;

#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct EmbeddedServerDescription {
	pub(crate) id: ServerId,
	game_host: [u8; 4],
	game_port: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ResultCode {
	OK = 0,
	InternalError = 1,
	IpVersionNot4 = 2,
}

#[no_mangle]
pub extern "C" fn run_new_server(result: &mut EmbeddedServerDescription) -> ResultCode {
	let mut registry = REGISTRY.lock().unwrap();
	registry.next_server_id += 1;
	let server_id = registry.next_server_id;

	match EmbeddedServerWrapper::run_new_server() {
		Ok(server) => {
			result.id = server_id;
			result.game_host = match server.game_socket_addr.ip() {
				IpAddr::V4(v4) => v4.octets(),
				IpAddr::V6(_) => return ResultCode::IpVersionNot4,
			};
			result.game_port = server.game_socket_addr.port();
			registry.servers.insert(server_id, server);
			ResultCode::OK
		}
		Err(_) => ResultCode::InternalError,
	}
}

#[no_mangle]
pub extern "C" fn destroy_server(server_id: ServerId) -> bool {
	let mut registry = REGISTRY.lock().unwrap();
	if let Some(server) = registry.servers.remove(&server_id) {
		server.shutdown();
		true
	} else {
		false
	}
}

#[cfg(test)]
mod test {
	use crate::ffi::server::{destroy_server, run_new_server, EmbeddedServerDescription, ResultCode};

	#[test]
	pub fn should_run_new_server() {
		let mut result = EmbeddedServerDescription::default();
		let result = run_new_server(&mut result);
		assert_eq!(result, ResultCode::OK)
	}

	#[test]
	pub fn should_destroy_server() {
		let mut result = EmbeddedServerDescription::default();
		run_new_server(&mut result);
		assert!(destroy_server(result.id));
		assert!(!destroy_server(result.id))
	}
}
