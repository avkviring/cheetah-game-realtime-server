use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;

pub mod channel;
pub mod client;
pub mod emulator;

pub fn bind_to_free_socket() -> std::io::Result<(UdpSocket, SocketAddr)> {
	for port in 2048..8912 {
		let socket_addr = SocketAddr::from_str(format!("0.0.0.0:{:}", port).as_str()).unwrap();
		match UdpSocket::bind(socket_addr) {
			Ok(socket) => {
				tracing::info!("[bind_to_free_socket] bind({:?})", socket_addr);
				return Result::Ok((socket, socket_addr));
			}
			Err(e) => match e.kind() {
				ErrorKind::AddrInUse => {}
				_ => return Err(e),
			},
		}
	}

	tracing::error!("[bind_to_free_socket] cannot find free socket");
	Result::Err(std::io::Error::new(ErrorKind::AddrInUse, "Check all ports in 2048..8912"))
}
