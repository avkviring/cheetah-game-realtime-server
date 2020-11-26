use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;

pub mod client;

pub fn bind_to_free_socket() -> Result<(UdpSocket, SocketAddr), ()> {
	for port in 2048..8912 {
		let socket_addr = SocketAddr::from_str(format!("0.0.0.0:{:}", port).as_str()).unwrap();
		match UdpSocket::bind(socket_addr) {
			Ok(socket) => {
				log::info!("[bind_to_free_socket] bind({:?})", socket_addr);
				return Result::Ok((socket, socket_addr));
			}
			Err(_) => {}
		}
	}
	
	log::error!("[bind_to_free_socket] cannot find free socket");
	Result::Err(())
}