use std::cell::RefCell;
use std::hash::Hash;
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::rc::Rc;

///
/// Абстракция для канала данных
/// - две реализации - UDP и Stub (для тестов)
///
pub trait Transport<PeerAddress: Hash> {
	fn create_channel(&self, self_address: PeerAddress) -> Result<Rc<RefCell<dyn Channel<PeerAddress>>>, TransportError>;
}

pub trait Channel<PeerAddress> {
	fn send(&mut self, to: &PeerAddress, buffer: &[u8]) -> Result<usize, TransportError>;
	fn receive(&self, buffer: &mut [u8]) -> Result<(usize, PeerAddress), TransportError>;
}

pub enum TransportError {
	AddressInUse,
	InvalidInput,
	NoData,
	FatalError,
}


#[derive(Default)]
pub struct UDPTransport {}

impl From<std::io::Error> for TransportError {
	fn from(e: Error) -> Self {
		match e.kind() {
			ErrorKind::AddrInUse => { TransportError::AddressInUse }
			ErrorKind::AddrNotAvailable => { TransportError::AddressInUse }
			ErrorKind::InvalidInput => { TransportError::InvalidInput }
			ErrorKind::WouldBlock => { TransportError::NoData }
			_ => { TransportError::FatalError }
		}
	}
}

impl Transport<SocketAddr> for UDPTransport {
	fn create_channel(&self, self_address: SocketAddr) -> Result<Rc<RefCell<dyn Channel<SocketAddr>>>, TransportError> {
		let socket = UdpSocket::bind(self_address)?;
		socket.set_nonblocking(true)?;
		Result::Ok(Rc::new(RefCell::new(UdpChannel { socket })))
	}
}


struct UdpChannel {
	socket: UdpSocket
}

impl Channel<SocketAddr> for UdpChannel {
	fn send(&mut self, to: &SocketAddr, buffer: &[u8]) -> Result<usize, TransportError> {
		self.socket.send_to(buffer, to).map_err(TransportError::from)
	}
	
	fn receive(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), TransportError> {
		self.socket.recv_from(buf).map_err(TransportError::from)
	}
}