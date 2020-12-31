use std::io;
use std::net::{SocketAddr, UdpSocket};

use crate::network::bind_to_free_socket;

#[derive(Debug)]
pub struct NetworkChannel {
	socket: UdpSocket,
}

impl NetworkChannel {
	pub fn new() -> Result<Self, ()> {
		let socket = bind_to_free_socket()?.0;
		socket.set_nonblocking(true).map_err(|_| ())?;
		Result::Ok(Self { socket })
	}

	pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
		self.socket.recv(buf)
	}
	pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
		self.socket.send_to(buf, addr)
	}
}
