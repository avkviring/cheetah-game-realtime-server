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

#[no_mangle]
pub extern "C" fn run_new_server(result: &mut EmbeddedServerDescription, on_error: extern "C" fn(*const u16)) -> bool {
	let mut registry = REGISTRY.lock().unwrap();
	registry.next_server_id += 1;
	let server_id = registry.next_server_id;

	match EmbeddedServerWrapper::run_new_server() {
		Ok(server) => {
			result.id = server_id;
			result.game_host = match server.game_socket_addr.ip() {
				IpAddr::V4(v4) => v4.octets(),
				IpAddr::V6(_) => {
					on_error(widestring::U16CString::from_str("IPv6 not supported").unwrap().as_ptr());
					return false;
				}
			};
			result.game_port = server.game_socket_addr.port();
			registry.servers.insert(server_id, server);
			true
		}
		Err(e) => {
			let string = widestring::U16CString::from_str(format!("{:?}", e)).unwrap();
			on_error(string.as_ptr());
			false
		}
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
	use crate::ffi::server::{destroy_server, run_new_server, EmbeddedServerDescription};

	#[test]
	pub fn should_run_new_server() {
		let mut result = EmbeddedServerDescription::default();
		let success = run_new_server(&mut result, on_error);
		assert!(success);
	}

	#[test]
	pub fn should_destroy_server() {
		let mut result = EmbeddedServerDescription::default();
		let success = run_new_server(&mut result, on_error);
		assert!(success);
		assert!(destroy_server(result.id));
		assert!(!destroy_server(result.id));
	}

	pub extern "C" fn on_error(message: *const u16) {
		panic!("Fail create server with message {:?}", message)
	}
}
