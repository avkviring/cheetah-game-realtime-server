use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Cursor, Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::rc::Rc;
pub use std::time::Instant;

use cheetah_common::protocol::codec::cipher::Cipher;
use cheetah_common::protocol::disconnect::command::DisconnectByCommandReason;
use cheetah_common::protocol::frame::headers::Header;
use cheetah_common::protocol::frame::input::InFrame;
use cheetah_common::protocol::frame::{FrameId, MAX_FRAME_SIZE};
use cheetah_common::protocol::others::member_id::MemberAndRoomId;
use cheetah_common::protocol::Protocol;
use cheetah_common::room::{MemberPrivateKey, RoomId, RoomMemberId};

use crate::room::template::config::MemberTemplate;
use crate::server::measurers::Measurers;
use crate::server::rooms::Rooms;

pub struct NetworkServer {
	sessions: HashMap<MemberAndRoomId, MemberSession>,
	socket: UdpSocket,
	measurers: Rc<RefCell<Measurers>>,
	start_application_time: Instant,
}

#[derive(Debug)]
struct MemberSession {
	peer_address: Option<SocketAddr>,
	private_key: MemberPrivateKey,
	last_receive_frame_id: FrameId,
	pub(crate) protocol: Protocol,
}

impl NetworkServer {
	pub fn new(socket: UdpSocket, measurers: Rc<RefCell<Measurers>>) -> Result<Self, Error> {
		socket.set_nonblocking(true)?;
		Ok(Self {
			sessions: Default::default(),
			socket,
			measurers,
			start_application_time: Instant::now(),
		})
	}

	pub fn cycle(&mut self, rooms: &mut Rooms, now: Instant) {
		self.receive(rooms, now);
		self.send(rooms);

		let mut disconnected = heapless::Vec::<MemberAndRoomId, 1000>::new();
		self.sessions.iter_mut().for_each(|(id, session)| {
			if session.protocol.is_disconnected(now).is_some() && !disconnected.is_full() {
				if let Err(e) = rooms.member_disconnected(id) {
					e.log_error(id.room_id, id.member_id);
				}
				disconnected.push(*id).unwrap();
			}
		});
		for id in disconnected {
			self.sessions.remove(&id);
		}
		self.measurers.borrow_mut().on_network_cycle(self.sessions.values().map(|session| &session.protocol.rtt));
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
					if session.peer_address.is_some() {
						for command in commands {
							session.protocol.out_commands_collector.add_command(command.channel_type, command.command.clone());
						}
						Self::send_frame(&self.socket, session);
					}
				}
			}
		});
	}

	fn send_frame(socket: &UdpSocket, session: &mut MemberSession) {
		if let (Some(peer_address), Some(frame)) = (session.peer_address, session.protocol.build_next_frame(Instant::now())) {
			let mut buffer = [0; MAX_FRAME_SIZE];
			let buffer_size = frame.encode(&mut Cipher::new(&session.private_key), &mut buffer).unwrap();
			match socket.send_to(&buffer[0..buffer_size], peer_address) {
				Ok(size) => {
					if size != buffer_size {
						tracing::error!("[network] size mismatch in socket.send_to {:?} {:?}", buffer.len(), size);
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

	fn receive(&mut self, rooms: &mut Rooms, now: Instant) {
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

	fn process_in_frame(&mut self, rooms: &mut Rooms, buffer: &[u8; MAX_FRAME_SIZE], size: usize, address: SocketAddr, now: Instant) {
		let start_time = Instant::now();
		let mut cursor = Cursor::new(&buffer[0..size]);
		match InFrame::decode_meta(&mut cursor) {
			Ok((connection_id, frame_id, headers)) => {
				let member_and_room_id_header: Option<MemberAndRoomId> = headers.first(Header::predicate_member_and_room_id).copied();

				match member_and_room_id_header {
					None => {
						tracing::error!("[network] MemberAndRoomId header not found {:?}", headers);
					}
					Some(member_and_room_id) => {
						match self.sessions.get_mut(&member_and_room_id) {
							None => {
								tracing::error!("[network] member session not found {:?}", member_and_room_id);
							}
							Some(session) => {
								let private_key = &session.private_key;
								match InFrame::decode_commands(true, frame_id, cursor, Cipher::new(private_key)) {
									Ok(commands) => {
										tracing::debug!("c2s {:?}", commands);
										let frame = InFrame::new(connection_id, frame_id, headers, commands);
										if frame.frame_id > session.last_receive_frame_id || session.last_receive_frame_id == 0 {
											session.peer_address.replace(address);
											session.last_receive_frame_id = frame.frame_id;
										}
										session.protocol.on_frame_received(&frame, now);
										rooms.execute_commands(member_and_room_id, session.protocol.in_commands_collector.get_ready_commands());
									}
									Err(e) => {
										tracing::error!("[network] error decode frame {:?}", e);
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

	pub fn register_member(&mut self, now: Instant, room_id: RoomId, member_id: RoomMemberId, template: MemberTemplate) {
		self.sessions.insert(
			MemberAndRoomId { member_id, room_id },
			MemberSession {
				peer_address: Default::default(),
				private_key: template.private_key,
				last_receive_frame_id: 0,
				protocol: Protocol::new(
					0,
					now,
					self.start_application_time,
					self.measurers.borrow().retransmit_count.clone(),
					self.measurers.borrow().ack_sent.clone(),
				),
			},
		);
	}

	/// Послать `DisconnectHeader` пользователю и удалить сессию с сервера
	pub fn disconnect_members(&mut self, member_and_room_ids: impl Iterator<Item = MemberAndRoomId>, reason: DisconnectByCommandReason) {
		for id in member_and_room_ids {
			if let Some(mut session) = self.sessions.remove(&id) {
				session.protocol.disconnect_by_command.disconnect(reason);
				Self::send_frame(&self.socket, &mut session);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::net::SocketAddr;
	use std::rc::Rc;
	use std::str::FromStr;
	use std::time::Instant;

	use cheetah_common::network::bind_to_free_socket;
	use cheetah_common::protocol::codec::cipher::Cipher;
	use cheetah_common::protocol::disconnect::command::DisconnectByCommandReason;
	use cheetah_common::protocol::frame::headers::Header;
	use cheetah_common::protocol::frame::output::OutFrame;
	use cheetah_common::protocol::frame::MAX_FRAME_SIZE;
	use cheetah_common::protocol::others::member_id::MemberAndRoomId;

	use crate::room::member::RoomMember;
	use crate::room::template::config::MemberTemplate;
	use crate::server::measurers::Measurers;
	use crate::server::network::NetworkServer;
	use crate::server::rooms::Rooms;

	#[test]
	fn should_not_panic_when_wrong_in_data() {
		let mut udp_server = create_network_layer();
		let mut rooms = Rooms::default();
		let buffer = [0; MAX_FRAME_SIZE];
		let usize = 100_usize;
		udp_server.process_in_frame(&mut rooms, &buffer, usize, SocketAddr::from_str("127.0.0.1:5002").unwrap(), Instant::now());
	}

	#[test]
	fn should_not_panic_when_wrong_member() {
		let mut udp_server = create_network_layer();
		let mut rooms = Rooms::default();
		let mut buffer = [0; MAX_FRAME_SIZE];
		let mut frame = OutFrame::new(0, 0);
		frame.headers.add(Header::MemberAndRoomId(MemberAndRoomId { member_id: 0, room_id: 0 }));
		let size = frame.encode(&mut Cipher::new(&[0; 32].as_slice().into()), &mut buffer).unwrap();
		udp_server.process_in_frame(&mut rooms, &buffer, size, SocketAddr::from_str("127.0.0.1:5002").unwrap(), Instant::now());
	}

	#[test]
	fn should_not_panic_when_missing_member_header() {
		let mut udp_server = create_network_layer();
		let mut rooms = Rooms::default();
		let mut buffer = [0; MAX_FRAME_SIZE];
		let frame = OutFrame::new(0, 0);
		let size = frame.encode(&mut Cipher::new(&[0; 32].as_slice().into()), &mut buffer).unwrap();
		udp_server.process_in_frame(&mut rooms, &buffer, size, SocketAddr::from_str("127.0.0.1:5002").unwrap(), Instant::now());
	}

	///
	/// Проверяем что адрес пользователя не будет переписан фреймом из прошлого
	///
	#[test]
	fn should_keep_address_from_last_frame() {
		let mut udp_server = create_network_layer();
		let mut rooms = Rooms::default();
		let mut buffer = [0; MAX_FRAME_SIZE];

		let member_template = MemberTemplate::new_member(Default::default(), Default::default());
		let member = RoomMember {
			id: 100,
			connected: false,
			attached: false,
			template: member_template.clone(),
			out_commands: Default::default(),
		};
		udp_server.register_member(Instant::now(), 0, member.id, member.template.clone());

		let mut frame = OutFrame::new(0, 100);
		let member_and_room_id = MemberAndRoomId { member_id: member.id, room_id: 0 };
		frame.headers.add(Header::MemberAndRoomId(member_and_room_id));
		let size = frame.encode(&mut Cipher::new(&member_template.private_key), &mut buffer).unwrap();

		let addr_1 = SocketAddr::from_str("127.0.0.1:5002").unwrap();
		let addr_2 = SocketAddr::from_str("127.0.0.1:5003").unwrap();

		udp_server.process_in_frame(&mut rooms, &buffer, size, addr_1, Instant::now());

		let mut frame = OutFrame::new(0, 10);
		frame.headers.add(Header::MemberAndRoomId(member_and_room_id));
		let size = frame.encode(&mut Cipher::new(&member_template.private_key), &mut buffer).unwrap();
		udp_server.process_in_frame(&mut rooms, &buffer, size, addr_2, Instant::now());

		assert_eq!(udp_server.sessions[&member_and_room_id].peer_address.unwrap(), addr_1);
	}

	#[test]
	fn should_disconnect_members() {
		let mut udp_server = create_network_layer();
		let member_template = MemberTemplate::new_member(Default::default(), Default::default());
		let member_to_delete = MemberAndRoomId { member_id: 0, room_id: 0 };
		udp_server.register_member(Instant::now(), member_to_delete.room_id, member_to_delete.member_id, member_template.clone());
		udp_server.register_member(Instant::now(), 0, 1, member_template);

		udp_server.disconnect_members(vec![member_to_delete].into_iter(), DisconnectByCommandReason::MemberDeleted);

		assert!(!udp_server.sessions.contains_key(&member_to_delete), "session should be deleted");
	}

	fn create_network_layer() -> NetworkServer {
		NetworkServer::new(bind_to_free_socket().unwrap(), Rc::new(RefCell::new(Measurers::new(prometheus::default_registry())))).unwrap()
	}
}
