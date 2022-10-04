use std::net::UdpSocket;

pub mod channel;
pub mod client;
pub mod emulator;

pub fn bind_to_free_socket() -> std::io::Result<UdpSocket> {
	UdpSocket::bind("127.0.0.1:0")
}
