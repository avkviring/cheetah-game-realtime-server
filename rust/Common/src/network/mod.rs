use std::collections::VecDeque;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::time::Instant;

use cheetah_protocol::codec::cipher::Cipher;
use cheetah_protocol::frame::disconnected_reason::DisconnectedReason;
use cheetah_protocol::frame::headers::Header;
use cheetah_protocol::frame::member_private_key::MemberPrivateKey;
use cheetah_protocol::frame::{ConnectionId, Frame};
use cheetah_protocol::others::member_id::MemberAndRoomId;
use cheetah_protocol::{Protocol, RoomId, RoomMemberId};

use crate::network::collectors::in_collector::InCommandsCollector;
use crate::network::collectors::out_collector::OutCommandsCollector;
use crate::network::socket::UdpSocketWrapper;

pub mod collectors;
pub mod emulator;
pub mod socket;

pub fn bind_to_free_socket() -> std::io::Result<UdpSocket> {
	UdpSocket::bind("0.0.0.0:0")
}

pub type CheetahProtocol = Protocol<InCommandsCollector, OutCommandsCollector>;

#[derive(Debug)]
pub struct NetworkChannel {
	pub state: ConnectionStatus,
	pub current_connection_id: ConnectionId,
	pub protocol: CheetahProtocol,
	private_key: MemberPrivateKey,
	server_address: SocketAddr,
	pub socket_wrapper: UdpSocketWrapper,
	out_frames: VecDeque<Frame>,
	member_and_room_id: MemberAndRoomId,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConnectionStatus {
	Connecting,
	///
	/// Соединение установлено
	///
	Connected,
	///
	/// Соединение закрыто
	///
	Disconnected(DisconnectedReason),
}

impl NetworkChannel {
	#[allow(clippy::unwrap_in_result)]
	pub fn new(
		connection_id: ConnectionId,
		server_side: bool,
		private_key: MemberPrivateKey,
		member_id: RoomMemberId,
		room_id: RoomId,
		server_address: SocketAddr,
		start_application_time: Instant,
	) -> std::io::Result<NetworkChannel> {
		let protocol = Protocol::new(InCommandsCollector::new(server_side), Default::default(), connection_id, Instant::now(), start_application_time);
		let channel = UdpSocketWrapper::new()?;

		Ok(NetworkChannel {
			current_connection_id: 0,
			state: ConnectionStatus::Connecting,
			protocol,
			private_key,
			server_address,
			socket_wrapper: channel,
			out_frames: Default::default(),
			member_and_room_id: MemberAndRoomId { member_id, room_id },
		})
	}

	pub fn cycle(&mut self, now: Instant) {
		if let ConnectionStatus::Disconnected(_) = self.state {
			return;
		}

		self.socket_wrapper.cycle(now);
		self.do_read(now);
		self.do_write(now);

		if self.protocol.is_connected(now) {
			self.state = ConnectionStatus::Connected;
		}

		if let Some(reason) = self.protocol.is_disconnected(now) {
			self.state = ConnectionStatus::Disconnected(reason);
		}
	}

	fn do_write(&mut self, now: Instant) {
		self.protocol.collect_out_frames(now, &mut self.out_frames);

		let mut buffer = [0; 2048];
		while let Some(frame) = self.out_frames.back_mut() {
			frame.headers.add_if_not_present(Header::MemberAndRoomId(self.member_and_room_id));

			let frame_buffer_size = frame.encode(&mut Cipher::new(&self.private_key), &mut buffer).unwrap();
			match self.socket_wrapper.send_to(now, &buffer[0..frame_buffer_size], self.server_address) {
				Ok(size) => {
					if size == frame_buffer_size {
						self.out_frames.pop_back();
					} else {
						tracing::error!("error send frame size mismatch send {:?}, frame {:?}", size, frame_buffer_size);
					}
				}
				Err(e) => {
					if e.kind() == ErrorKind::WouldBlock {
					} else {
						tracing::error!("error send {:?}", e);
						self.state = ConnectionStatus::Disconnected(DisconnectedReason::IOError(format!("error send {e:?}")));
					}
				}
			}
		}
	}

	fn do_read(&mut self, now: Instant) {
		let mut buffer = [0; 2048];
		loop {
			match self.socket_wrapper.recv(now, &mut buffer) {
				Err(e) => {
					if e.kind() == ErrorKind::WouldBlock {
					} else {
						tracing::error!("error receive {:?}", e);
						self.state = ConnectionStatus::Disconnected(DisconnectedReason::IOError(format!("error receive {e:?}")));
					}
					break;
				}
				Ok(size) => {
					let cipher = Some(Cipher::new(&self.private_key));
					let frame_decode_result = Frame::decode(&buffer[0..size], |_| cipher);
					match frame_decode_result {
						Ok(frame) => {
							self.on_frame_received(now, &frame);
						}
						Err(e) => {
							tracing::error!("error decode frame {:?}", e);
						}
					}
				}
			}
		}
	}

	fn on_frame_received(&mut self, now: Instant, frame: &Frame) {
		self.protocol.on_frame_received(frame, now);
	}
}
