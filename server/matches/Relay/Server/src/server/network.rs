use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Cursor, Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::rc::Rc;
use std::time::Instant;

use cheetah_matches_relay_common::protocol::codec::cipher::Cipher;
use cheetah_matches_relay_common::protocol::frame::headers::Header;
use cheetah_matches_relay_common::protocol::frame::input::InFrame;
use cheetah_matches_relay_common::protocol::frame::{FrameId, MAX_FRAME_SIZE};
use cheetah_matches_relay_common::protocol::others::user_id::MemberAndRoomId;
use cheetah_matches_relay_common::protocol::Protocol;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId, UserPrivateKey};

use crate::room::template::config::MemberTemplate;
use crate::server::measurers::Measurers;
use crate::server::rooms::Rooms;

pub struct NetworkLayer {
	sessions: HashMap<MemberAndRoomId, MemberSession>,
	socket: UdpSocket,
	measurers: Rc<RefCell<Measurers>>,
	start_application_time: Instant,
}

#[derive(Debug)]
struct MemberSession {
	peer_address: Option<SocketAddr>,
	private_key: UserPrivateKey,
	max_receive_frame_id: FrameId,
	pub protocol: Protocol,
}

impl NetworkLayer {
	pub fn new(socket: UdpSocket, measurers: Rc<RefCell<Measurers>>) -> Result<Self, Error> {
		socket.set_nonblocking(true)?;
		tracing::info!("Starting network server on {:?}", socket);
		Result::Ok(Self {
			sessions: Default::default(),
			socket,
			measurers,
			start_application_time: Instant::now(),
		})
	}

	pub fn cycle(&mut self, rooms: &mut Rooms, now: &Instant) {
		self.receive(rooms, now);
		self.send(rooms);

		let mut disconnected = heapless::Vec::<MemberAndRoomId, 1000>::new();
		self.sessions.iter_mut().for_each(|(id, session)| {
			if session.protocol.is_disconnected(now).is_some() && !disconnected.is_full() {
				if let Err(e) = rooms.user_disconnected(id) {
					e.log_error(id.room_id, id.member_id);
				}
				disconnected.push(id.clone()).unwrap();
			}
		});
		for id in disconnected {
			self.sessions.remove(&id);
		}
	}

	///
	/// Отправить команды клиентам
	///
	fn send(&mut self, rooms: &mut Rooms) {
		rooms.collect_out_commands(|room_id, member_id, commands| {
			let id = MemberAndRoomId {
				member_id: *member_id,
				room_id: *room_id,
			};
			match self.sessions.get_mut(&id) {
				None => {
					tracing::error!("[network] member not found {:?}", id);
				}
				Some(session) => {
					if let Some(peer_address) = session.peer_address.as_ref() {
						for command in commands {
							session
								.protocol
								.out_commands_collector
								.add_command(command.channel_type.clone(), command.command.clone());
						}
						if let Some(frame) = session.protocol.build_next_frame(&Instant::now()) {
							let mut buffer = [0; MAX_FRAME_SIZE];
							let buffer_size = frame.encode(&mut Cipher::new(&session.private_key), &mut buffer).unwrap();
							match self.socket.send_to(&buffer[0..buffer_size], peer_address) {
								Ok(size) => {
									if size != buffer_size {
										tracing::error!(
											"[network] size mismatch in socket.send_to {:?} {:?}",
											buffer.len(),
											size
										);
									}
								}
								Err(e) => match e.kind() {
									ErrorKind::WouldBlock => {}
									_ => {
										tracing::error!("[network] socket error {:?}", e);
									}
								},
							}
						}
					}
				}
			}
		});
	}

	fn receive(&mut self, rooms: &mut Rooms, now: &Instant) {
		let mut buffer = [0; MAX_FRAME_SIZE];
		loop {
			let result = self.socket.recv_from(&mut buffer);
			match result {
				Ok((size, address)) => self.process_in_frame(rooms, &buffer, size, address, now),
				Err(e) => match e.kind() {
					ErrorKind::WouldBlock => {
						return;
					}
					_ => {
						tracing::error!("[network] error in socket.recv_from {:?}", e);
					}
				},
			}
		}
	}

	fn process_in_frame(
		&mut self,
		rooms: &mut Rooms,
		buffer: &[u8; MAX_FRAME_SIZE],
		size: usize,
		address: SocketAddr,
		now: &Instant,
	) {
		let start_time = Instant::now();
		let mut cursor = Cursor::new(&buffer[0..size]);
		match InFrame::decode_headers(&mut cursor) {
			Ok((frame_id, headers)) => {
				let member_and_room_id_header: Option<MemberAndRoomId> =
					headers.first(Header::predicate_member_and_room_id).cloned();

				match member_and_room_id_header {
					None => {
						tracing::error!("[network] MemberAndRoomId header not found {:?}", headers);
					}
					Some(user_and_room_id) => {
						match self.sessions.get_mut(&user_and_room_id) {
							None => {
								tracing::error!("[network] user session not found {:?}", user_and_room_id);
							}
							Some(session) => {
								let private_key = &session.private_key;
								match InFrame::decode_frame_commands(true, frame_id, cursor, Cipher::new(private_key)) {
									Ok(commands) => {
										let frame = InFrame::new(frame_id, headers, commands);
										if frame.frame_id > session.max_receive_frame_id || session.max_receive_frame_id == 0 {
											session.peer_address.replace(address);
											session.max_receive_frame_id = frame.frame_id;
										}
										session.protocol.on_frame_received(frame, now);
										rooms.execute_commands(
											user_and_room_id,
											session.protocol.in_commands_collector.get_ready_commands(),
										)
									}
									Err(e) => {
										tracing::error!("[network] error decode frame {:?}", e)
									}
								}
							}
						};
					}
				}
			}
			Err(e) => {
				tracing::error!("decode headers error {:?}", e);
			}
		}

		let mut measurers = self.measurers.borrow_mut();
		measurers.on_income_frame(size, start_time.elapsed());
	}

	pub fn register_user(&mut self, now: &Instant, room_id: RoomId, user_id: RoomMemberId, template: MemberTemplate) {
		self.sessions.insert(
			MemberAndRoomId {
				member_id: user_id,
				room_id,
			},
			MemberSession {
				peer_address: Default::default(),
				private_key: template.private_key,
				max_receive_frame_id: 0,
				protocol: Protocol::new(now, &self.start_application_time),
			},
		);
	}
}

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::net::SocketAddr;
	use std::rc::Rc;
	use std::str::FromStr;
	use std::time::Instant;

	use cheetah_matches_relay_common::network::bind_to_free_socket;
	use cheetah_matches_relay_common::protocol::codec::cipher::Cipher;
	use cheetah_matches_relay_common::protocol::frame::headers::Header;
	use cheetah_matches_relay_common::protocol::frame::output::OutFrame;
	use cheetah_matches_relay_common::protocol::frame::MAX_FRAME_SIZE;
	use cheetah_matches_relay_common::protocol::others::user_id::MemberAndRoomId;

	use crate::room::template::config::MemberTemplate;
	use crate::room::Member;
	use crate::server::measurers::Measurers;
	use crate::server::network::NetworkLayer;
	use crate::server::rooms::Rooms;

	#[test]
	fn should_not_panic_when_wrong_in_data() {
		let mut udp_server = create_network_layer();
		let mut rooms = Rooms::new(Rc::new(RefCell::new(Measurers::new(prometheus::default_registry()))));
		let buffer = [0; MAX_FRAME_SIZE];
		let usize = 100_usize;
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
		let mut udp_server = create_network_layer();
		let mut rooms = Rooms::new(Rc::new(RefCell::new(Measurers::new(prometheus::default_registry()))));
		let mut buffer = [0; MAX_FRAME_SIZE];
		let mut frame = OutFrame::new(0);
		frame.headers.add(Header::MemberAndRoomId(MemberAndRoomId {
			member_id: 0,
			room_id: 0,
		}));
		let size = frame.encode(&mut Cipher::new(&[0; 32]), &mut buffer).unwrap();
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
		let mut udp_server = create_network_layer();
		let mut rooms = Rooms::new(Rc::new(RefCell::new(Measurers::new(prometheus::default_registry()))));
		let mut buffer = [0; MAX_FRAME_SIZE];
		let frame = OutFrame::new(0);
		let size = frame.encode(&mut Cipher::new(&[0; 32]), &mut buffer).unwrap();
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
		let mut udp_server = create_network_layer();
		let mut rooms = Rooms::new(Rc::new(RefCell::new(Measurers::new(prometheus::default_registry()))));
		let mut buffer = [0; MAX_FRAME_SIZE];

		let user_template = MemberTemplate {
			private_key: Default::default(),
			groups: Default::default(),
			objects: Default::default(),
		};
		let user = Member {
			id: 100,
			connected: false,
			attached: false,
			template: user_template.clone(),
			compare_and_set_cleaners: Default::default(),
			out_commands: Default::default(),
		};
		udp_server.register_user(&Instant::now(), 0, user.id, user.template.clone());

		let mut frame = OutFrame::new(100);
		let user_and_room_id = MemberAndRoomId {
			member_id: user.id,
			room_id: 0,
		};
		frame.headers.add(Header::MemberAndRoomId(user_and_room_id.clone()));
		let size = frame
			.encode(&mut Cipher::new(&user_template.private_key), &mut buffer)
			.unwrap();

		let addr_1 = SocketAddr::from_str("127.0.0.1:5002").unwrap();
		let addr_2 = SocketAddr::from_str("127.0.0.1:5003").unwrap();

		udp_server.process_in_frame(&mut rooms, &buffer, size, addr_1, &Instant::now());

		let mut frame = OutFrame::new(10);
		frame.headers.add(Header::MemberAndRoomId(user_and_room_id.clone()));
		let size = frame
			.encode(&mut Cipher::new(&user_template.private_key), &mut buffer)
			.unwrap();
		udp_server.process_in_frame(&mut rooms, &buffer, size, addr_2, &Instant::now());

		assert_eq!(
			udp_server.sessions.get(&user_and_room_id).unwrap().peer_address.unwrap(),
			addr_1
		);
	}

	fn create_network_layer() -> NetworkLayer {
		NetworkLayer::new(
			bind_to_free_socket().unwrap().0,
			Rc::new(RefCell::new(Measurers::new(prometheus::default_registry()))),
		)
		.unwrap()
	}
}
