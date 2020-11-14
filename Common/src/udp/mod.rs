use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;

pub mod client;

pub fn bind_to_free_socket() -> Result<(UdpSocket, SocketAddr), ()> {
	for port in 2048..8912 {
		let socket_addr = SocketAddr::from_str(format!("0.0.0.0:{:}", port).as_str()).unwrap();
		match UdpSocket::bind(socket_addr) {
			Ok(socket) => {
				return Result::Ok((socket, socket_addr));
			}
			Err(_) => {}
		}
	}
	
	Result::Err(())
}