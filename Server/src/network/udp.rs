use std::collections::{HashMap, VecDeque};
use std::io::{Cursor, Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};

use fnv::{FnvBuildHasher, FnvHashMap};

use cheetah_relay_common::protocol::codec::cipher::Cipher;
use cheetah_relay_common::protocol::frame::{Frame, FrameId};
use cheetah_relay_common::protocol::frame::headers::Header;
use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};

use crate::rooms::{OutFrame, Rooms};

#[derive(Debug)]
pub struct UDPServer {
	sessions: HashMap<UserPublicKey, UserSession, FnvBuildHasher>,
	socket: UdpSocket,
	halt: bool,
	tmp_out_frames: VecDeque<OutFrame>,
}


#[derive(Debug)]
struct UserSession {
	peer_address: Option<SocketAddr>,
	private_key: UserPrivateKey,
	max_receive_frame_id: FrameId,
}

impl UDPServer {
	pub fn new(address: SocketAddr) -> Result<Self, Error> {
		let socket = UdpSocket::bind(address)?;
		socket.set_nonblocking(true).unwrap();
		Result::Ok(
			Self {
				sessions: FnvHashMap::default(),
				socket,
				halt: false,
				tmp_out_frames: VecDeque::with_capacity(50_000),
			})
	}
	
	pub fn register_user(&mut self, public_key: UserPublicKey, private_key: UserPrivateKey) {
		self.sessions.insert(public_key, UserSession {
			peer_address: Default::default(),
			private_key,
			max_receive_frame_id: 0,
		});
	}
	
	pub fn cycle(&mut self, rooms: &mut Rooms) {
		self.receive(rooms);
		self.send(rooms)
	}
	
	
	fn send(&mut self, rooms: &mut Rooms) {
		rooms.collect_out_frames(&mut self.tmp_out_frames);
		let mut buffer = [0; 2048];
		
		while let Some(OutFrame { user_public_key, frame }) = self.tmp_out_frames.back_mut() {
			match self.sessions.get(&user_public_key) {
				None => {}
				Some(session) => {
					let (commands, buffer_size) = frame.encode(&mut Cipher::new(&session.private_key), &mut buffer);
					rooms.return_commands(&user_public_key, commands);
					match self.socket.send_to(&buffer[0..buffer_size], session.peer_address.unwrap()) {
						Ok(size) => {
							if size != buffer.len() {
								log::error!("panic - size mismatch in socket.send_to {:?} {:?}", buffer.len(), size);
							}
							self.tmp_out_frames.pop_back();
						}
						Err(e) => {
							if let ErrorKind::WouldBlock = e.kind() {
								return;
							}
						}
					}
				}
			}
		}
	}
	
	fn receive(&mut self, rooms: &mut Rooms) {
		let mut buffer = [0; 2048];
		loop {
			let result = self.socket.recv_from(&mut buffer);
			match result {
				Ok((size, address)) => self.process_in_frame(rooms, &mut buffer, size, address),
				Err(e) => {
					match e.kind() {
						ErrorKind::WouldBlock => {
							return;
						}
						_ => {
							log::error!("error in socket.recv_from {:?}", e);
						}
					}
				}
			}
		}
	}
	
	fn process_in_frame(&mut self, rooms: &mut Rooms, buffer: &[u8; 2048], size: usize, address: SocketAddr) {
		let mut cursor = Cursor::new(&buffer[0..size]);
		match Frame::decode_headers(&mut cursor) {
			Ok((frame_header, headers)) => {
				let user_public_key_header: Option<UserPublicKey> = headers.first(Header::predicate_user_public_key).cloned();
				match user_public_key_header {
					None => {
						log::error!("user public key not found");
					}
					Some(public_key) => {
						match self.sessions.get_mut(&public_key) {
							None => {
								log::error!("user session not found for key {:?}", public_key);
							}
							Some(session) => {
								let private_key = &session.private_key;
								match Frame::decode_frame(cursor, Cipher::new(private_key), frame_header, headers) {
									Ok(frame) => {
										if frame.header.frame_id > session.max_receive_frame_id || session.max_receive_frame_id == 0 {
											session.peer_address.replace(address);
											session.max_receive_frame_id = frame.header.frame_id;
										}
										rooms.on_frame_received(&public_key, frame);
									}
									Err(e) => {
										log::error!("error decode frame {:?}", e)
									}
								}
							}
						}
					}
				}
			}
			Err(e) => {
				log::error!("decode headers error {:?}", e);
			}
		}
	}
}


#[cfg(test)]
mod tests {
	use std::net::SocketAddr;
	use std::str::FromStr;
	
	use cheetah_relay_common::protocol::codec::cipher::Cipher;
	use cheetah_relay_common::protocol::frame::Frame;
	use cheetah_relay_common::protocol::frame::headers::Header;
	
	use crate::network::udp::UDPServer;
	use crate::rooms::Rooms;
	
	#[test]
	fn should_not_panic_when_wrong_in_data() {
		let mut udp_server = UDPServer::new(SocketAddr::from_str("127.0.0.1:5001").unwrap()).unwrap();
		let mut rooms = Rooms::default();
		let buffer = [0; 2048];
		let usize = 100 as usize;
		udp_server.process_in_frame(&mut rooms, &buffer, usize, SocketAddr::from_str("127.0.0.1:5002").unwrap());
	}
	
	#[test]
	fn should_not_panic_when_wrong_user() {
		let mut udp_server = UDPServer::new(SocketAddr::from_str("127.0.0.1:5002").unwrap()).unwrap();
		let mut rooms = Rooms::default();
		let mut buffer = [0; 2048];
		let mut frame = Frame::new(0);
		frame.headers.add(Header::UserPublicKey(0));
		let size = frame.encode(&mut Cipher::new(&[0; 32]), &mut buffer).1;
		udp_server.process_in_frame(&mut rooms, &buffer, size, SocketAddr::from_str("127.0.0.1:5002").unwrap());
	}
	
	#[test]
	fn should_not_panic_when_missing_user_header() {
		let mut udp_server = UDPServer::new(SocketAddr::from_str("127.0.0.1:5003").unwrap()).unwrap();
		let mut rooms = Rooms::default();
		let mut buffer = [0; 2048];
		let mut frame = Frame::new(0);
		let size = frame.encode(&mut Cipher::new(&[0; 32]), &mut buffer).1;
		udp_server.process_in_frame(&mut rooms, &buffer, size, SocketAddr::from_str("127.0.0.1:5002").unwrap());
	}
	
	
	///
	/// Проверяем что адрес пользователя не будет переписан фреймом из прошлого
	///
	#[test]
	fn should_keep_address_from_last_frame() {
		let mut udp_server = UDPServer::new(SocketAddr::from_str("127.0.0.1:5004").unwrap()).unwrap();
		let mut rooms = Rooms::default();
		let mut buffer = [0; 2048];
		
		
		let public_key = 0;
		let private_key = [0; 32];
		
		udp_server.register_user(public_key, private_key);
		
		let mut frame = Frame::new(100);
		frame.headers.add(Header::UserPublicKey(public_key));
		let size = frame.encode(&mut Cipher::new(&private_key), &mut buffer).1;
		
		let addr_1 = SocketAddr::from_str("127.0.0.1:5002").unwrap();
		let addr_2 = SocketAddr::from_str("127.0.0.1:5003").unwrap();
		
		udp_server.process_in_frame(&mut rooms, &buffer, size, addr_1);
		
		
		let mut frame = Frame::new(10);
		frame.headers.add(Header::UserPublicKey(public_key));
		let size = frame.encode(&mut Cipher::new(&private_key), &mut buffer).1;
		udp_server.process_in_frame(&mut rooms, &buffer, size, addr_2);
		
		assert_eq!(udp_server.sessions.get(&public_key).unwrap().peer_address.unwrap(), addr_1);
	}
}