use std::net::{IpAddr, SocketAddr};

use crate::ffi::{ServerId, REGISTRY};
use crate::EmbeddedServerWrapper;

#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub(crate) struct EmbeddedServerDescription {
	pub(crate) id: ServerId,
	game_ip: [u8; 4],
	game_port: u16,
	internal_grpc_ip: [u8; 4],
	internal_grpc_port: u16,
	internal_webgrpc_ip: [u8; 4],
	internal_webgrpc_port: u16,
	admin_webgrpc_ip: [u8; 4],
	admin_webgrpc_port: u16,
}

#[repr(C)]
#[derive(Default, Debug)]
pub(crate) struct BindSocket {
	ip: [u8; 4],
	port: u16,
}

#[no_mangle]
pub(crate) extern "C" fn run_new_server(
	result: &mut EmbeddedServerDescription,
	on_error: extern "C" fn(*const u16),
	internal_grpc_socket: &BindSocket,
	internal_webgrpc_socket: &BindSocket,
	admin_webgrpc_socket: &BindSocket,
	game_udp_socket: &BindSocket,
) -> bool {
	let mut registry = REGISTRY.lock().unwrap();
	registry.next_server_id += 1;
	let server_id = registry.next_server_id;

	let internal_grpc_address = SocketAddr::new(IpAddr::from(internal_grpc_socket.ip), internal_grpc_socket.port);
	let internal_webgrpc_address = SocketAddr::new(IpAddr::from(internal_webgrpc_socket.ip), internal_webgrpc_socket.port);
	let admin_webgrpc_address = SocketAddr::new(IpAddr::from(admin_webgrpc_socket.ip), admin_webgrpc_socket.port);
	let game_udp_address = SocketAddr::new(IpAddr::from(game_udp_socket.ip), game_udp_socket.port);

	match EmbeddedServerWrapper::run_new_server(internal_grpc_address, internal_webgrpc_address, admin_webgrpc_address, game_udp_address) {
		Ok(server) => {
			result.id = server_id;

			if !set_addr(&mut result.game_ip, on_error, &server.game_socket_addr.ip())
				|| !set_addr(&mut result.internal_grpc_ip, on_error, &server.internal_grpc_socket_addr.ip())
				|| !set_addr(&mut result.internal_webgrpc_ip, on_error, &server.internal_webgrpc_socket_addr.ip())
				|| !set_addr(&mut result.admin_webgrpc_ip, on_error, &server.admin_webgrpc_socket_addr.ip())
			{
				return false;
			}

			result.game_port = server.game_socket_addr.port();
			result.internal_grpc_port = server.internal_grpc_socket_addr.port();
			result.internal_webgrpc_port = server.internal_webgrpc_socket_addr.port();
			result.admin_webgrpc_port = server.admin_webgrpc_socket_addr.port();
			registry.servers.insert(server_id, server);
			true
		}
		Err(e) => {
			let string = widestring::U16CString::from_str(format!("{e:?}")).unwrap();
			on_error(string.as_ptr());
			false
		}
	}
}

fn set_addr(out: &mut [u8; 4], on_error: extern "C" fn(*const u16), ip_addr: &IpAddr) -> bool {
	match ip_addr {
		IpAddr::V4(v4) => {
			*out = v4.octets();
			true
		}
		IpAddr::V6(_) => {
			on_error(widestring::U16CString::from_str("IPv6 not supported").unwrap().as_ptr());
			false
		}
	}
}

#[no_mangle]
pub(crate) extern "C" fn destroy_server(server_id: ServerId) -> bool {
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
	pub(crate) fn should_run_new_server() {
		let mut result = EmbeddedServerDescription::default();
		let success = setup_server(&mut result);
		assert!(success);
	}

	#[test]
	pub(crate) fn should_destroy_server() {
		let mut result = EmbeddedServerDescription::default();
		let success = setup_server(&mut result);
		assert!(success);
		assert!(destroy_server(result.id));
		assert!(!destroy_server(result.id));
	}

	pub(crate) extern "C" fn on_error(message: *const u16) {
		panic!("Fail create server with message {message:?}")
	}

	fn setup_server(mut result: &mut EmbeddedServerDescription) -> bool {
		run_new_server(&mut result, on_error, &Default::default(), &Default::default(), &Default::default(), &Default::default())
	}
}
