use std::collections::{HashMap, VecDeque};
use std::io::{Cursor, Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};

use cheetah_relay_common::protocol::codec::cipher::Cipher;
use cheetah_relay_common::protocol::frame::Frame;
use cheetah_relay_common::protocol::frame::headers::Header;

use crate::rooms::{OutFrame, Rooms};
use cheetah_relay_common::room::{UserPublicKey, UserPrivateKey};

#[derive(Debug)]
pub struct UDPServer {
	sessions: HashMap<UserPublicKey, UserSession>,
	socket: UdpSocket,
	halt: bool,
	tmp_out_frames: VecDeque<OutFrame>,
}


#[derive(Debug)]
struct UserSession {
	peer_address: Option<SocketAddr>,
	private_key: UserPrivateKey,
}

impl UDPServer {
	pub fn new(address: SocketAddr) -> Result<Self, Error> {
		let socket = UdpSocket::bind(address)?;
		socket.set_nonblocking(true).unwrap();
		Result::Ok(
			Self {
				sessions: Default::default(),
				socket,
				halt: false,
				tmp_out_frames: VecDeque::with_capacity(50_000),
			})
	}
	
	pub fn add_user(&mut self, public_key: UserPublicKey, private_key: UserPrivateKey) {
		self.sessions.insert(public_key, UserSession {
			peer_address: Default::default(),
			private_key,
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
				Ok((size, address)) => {
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
													session.peer_address.replace(address);
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
}