use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::io::{Cursor, Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::rc::Rc;
use std::time::Instant;

use fnv::FnvBuildHasher;

use cheetah_relay_common::protocol::codec::cipher::Cipher;
use cheetah_relay_common::protocol::frame::headers::Header;
use cheetah_relay_common::protocol::frame::{Frame, FrameId};
use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};

use crate::room::template::config::UserTemplate;
use crate::room::{RoomId, RoomUserListener};
use crate::rooms::{OutFrame, Rooms};

#[derive(Debug)]
pub struct UDPServer {
	sessions: Rc<RefCell<UserSessions>>,
	socket: UdpSocket,
	halt: bool,
	tmp_out_frames: VecDeque<OutFrame>,
}

#[derive(Default, Debug)]
struct UserSessions {
	sessions: HashMap<UserPublicKey, UserSession, FnvBuildHasher>,
}

#[derive(Debug)]
struct UserSession {
	peer_address: Option<SocketAddr>,
	private_key: UserPrivateKey,
	max_receive_frame_id: FrameId,
}

impl UDPServer {
	pub fn new(socket: UdpSocket) -> Result<Self, Error> {
		socket.set_nonblocking(true)?;
		log::info!("Starting udp server on {:?}", socket);
		Result::Ok(Self {
			sessions: Default::default(),
			socket,
			halt: false,
			tmp_out_frames: VecDeque::with_capacity(50_000),
		})
	}

	pub fn cycle(&mut self, rooms: &mut Rooms, now: &Instant) {
		self.receive(rooms, now);
		self.send(rooms, now);
	}

	fn send(&mut self, rooms: &mut Rooms, now: &Instant) {
		rooms.collect_out_frames(&mut self.tmp_out_frames, now);
		let mut buffer = [0; 2048];
		while let Some(OutFrame { user_public_key, frame }) = self.tmp_out_frames.back() {
			match self.sessions.clone().borrow().sessions.get(user_public_key) {
				None => {}
				Some(session) => {
					log::trace!("[udp] server -> user({:?}) {:?}", user_public_key, frame);
					let buffer_size = frame.encode(&mut Cipher::new(&session.private_key), &mut buffer);
					match self.socket.send_to(&buffer[0..buffer_size], session.peer_address.unwrap()) {
						Ok(size) => {
							if size == buffer_size {
								self.tmp_out_frames.pop_back();
							} else {
								log::error!("[udp] size mismatch in socket.send_to {:?} {:?}", buffer.len(), size);
							}
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

	fn receive(&mut self, rooms: &mut Rooms, now: &Instant) {
		let mut buffer = [0; Frame::MAX_FRAME_SIZE];
		loop {
			let result = self.socket.recv_from(&mut buffer);
			match result {
				Ok((size, address)) => self.process_in_frame(rooms, &mut buffer, size, address, now),
				Err(e) => match e.kind() {
					ErrorKind::WouldBlock => {
						return;
					}
					_ => {
						log::error!("[udp] error in socket.recv_from {:?}", e);
					}
				},
			}
		}
	}

	fn process_in_frame(&mut self, rooms: &mut Rooms, buffer: &[u8; Frame::MAX_FRAME_SIZE], size: usize, address: SocketAddr, now: &Instant) {
		let mut cursor = Cursor::new(&buffer[0..size]);
		match Frame::decode_headers(&mut cursor) {
			Ok((frame_header, headers)) => {
				let user_public_key_header: Option<UserPublicKey> = headers.first(Header::predicate_user_public_key).cloned();

				let sessions_cloned = self.sessions.clone();
				match user_public_key_header {
					None => {
						log::error!("[udp] user public key not found");
					}
					Some(user_public_key) => {
						let mut readed_frame = Option::None;
						match sessions_cloned.borrow_mut().sessions.get_mut(&user_public_key) {
							None => {
								log::error!("[udp] user session not found for key {:?}", user_public_key);
							}
							Some(session) => {
								let private_key = &session.private_key;
								match Frame::decode_frame(cursor, Cipher::new(private_key), frame_header, headers) {
									Ok(frame) => {
										if frame.header.frame_id > session.max_receive_frame_id || session.max_receive_frame_id == 0 {
											session.peer_address.replace(address);
											session.max_receive_frame_id = frame.header.frame_id;
										}
										readed_frame.replace(frame);
									}
									Err(e) => {
										log::error!("[udp] error decode frame {:?}", e)
									}
								}
							}
						};
						if let Some(frame) = readed_frame {
							rooms.on_frame_received(&user_public_key, frame, &now);
						}
					}
				}
			}
			Err(e) => {
				log::error!("decode headers error {:?}", e);
			}
		}
	}

	pub fn get_room_user_listener(&self) -> Rc<RefCell<dyn RoomUserListener>> {
		self.sessions.clone()
	}
}

impl RoomUserListener for UserSessions {
	fn register_user(&mut self, _: RoomId, template: &UserTemplate) {
		self.sessions.insert(
			template.public_key,
			UserSession {
				peer_address: Default::default(),
				private_key: template.private_key,
				max_receive_frame_id: 0,
			},
		);
	}

	fn connected_user(&mut self, _: RoomId, _: &UserTemplate) {}

	fn disconnected_user(&mut self, _: RoomId, template: &UserTemplate) {
		self.sessions.remove(&template.public_key);
	}
}

#[cfg(test)]
mod tests {
	use std::net::SocketAddr;
	use std::str::FromStr;
	use std::time::Instant;

	use cheetah_relay_common::protocol::codec::cipher::Cipher;
	use cheetah_relay_common::protocol::frame::headers::Header;
	use cheetah_relay_common::protocol::frame::Frame;
	use cheetah_relay_common::udp::bind_to_free_socket;

	use crate::network::udp::UDPServer;
	use crate::room::template::config::UserTemplate;
	use crate::room::RoomUserListener;
	use crate::rooms::Rooms;

	#[test]
	fn should_not_panic_when_wrong_in_data() {
		let mut udp_server = UDPServer::new(bind_to_free_socket().unwrap().0).unwrap();
		let mut rooms = Rooms::default();
		let buffer = [0; Frame::MAX_FRAME_SIZE];
		let usize = 100 as usize;
		udp_server.process_in_frame(
			&mut rooms,
			&buffer,
			usize,
			SocketAddr::from_str("127.0.0.1:5002").unwrap(),
			&Instant::now(),
		);
	}

	#[test]
	fn should_not_panic_when_wrong_user() {
		let mut udp_server = UDPServer::new(bind_to_free_socket().unwrap().0).unwrap();
		let mut rooms = Rooms::default();
		let mut buffer = [0; Frame::MAX_FRAME_SIZE];
		let mut frame = Frame::new(0);
		frame.headers.add(Header::UserPublicKey(0));
		let size = frame.encode(&mut Cipher::new(&[0; 32]), &mut buffer);
		udp_server.process_in_frame(
			&mut rooms,
			&buffer,
			size,
			SocketAddr::from_str("127.0.0.1:5002").unwrap(),
			&Instant::now(),
		);
	}

	#[test]
	fn should_not_panic_when_missing_user_header() {
		let mut udp_server = UDPServer::new(bind_to_free_socket().unwrap().0).unwrap();
		let mut rooms = Rooms::default();
		let mut buffer = [0; Frame::MAX_FRAME_SIZE];
		let frame = Frame::new(0);
		let size = frame.encode(&mut Cipher::new(&[0; 32]), &mut buffer);
		udp_server.process_in_frame(
			&mut rooms,
			&buffer,
			size,
			SocketAddr::from_str("127.0.0.1:5002").unwrap(),
			&Instant::now(),
		);
	}

	///
	/// Проверяем что адрес пользователя не будет переписан фреймом из прошлого
	///
	#[test]
	fn should_keep_address_from_last_frame() {
		let mut udp_server = UDPServer::new(bind_to_free_socket().unwrap().0).unwrap();
		let mut rooms = Rooms::default();
		let mut buffer = [0; Frame::MAX_FRAME_SIZE];

		let user_template = UserTemplate {
			public_key: Default::default(),
			private_key: Default::default(),
			access_groups: Default::default(),
			objects: Default::default(),
			unmapping: Default::default(),
		};
		udp_server.sessions.clone().borrow_mut().register_user(0, &user_template);

		let mut frame = Frame::new(100);
		frame.headers.add(Header::UserPublicKey(user_template.public_key));
		let size = frame.encode(&mut Cipher::new(&user_template.private_key), &mut buffer);

		let addr_1 = SocketAddr::from_str("127.0.0.1:5002").unwrap();
		let addr_2 = SocketAddr::from_str("127.0.0.1:5003").unwrap();

		udp_server.process_in_frame(&mut rooms, &buffer, size, addr_1, &Instant::now());

		let mut frame = Frame::new(10);
		frame.headers.add(Header::UserPublicKey(user_template.public_key));
		let size = frame.encode(&mut Cipher::new(&user_template.private_key), &mut buffer);
		udp_server.process_in_frame(&mut rooms, &buffer, size, addr_2, &Instant::now());

		assert_eq!(
			udp_server
				.sessions
				.clone()
				.borrow()
				.sessions
				.get(&user_template.public_key)
				.unwrap()
				.peer_address
				.unwrap(),
			addr_1
		);
	}
}
