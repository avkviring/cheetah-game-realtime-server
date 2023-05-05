use std::collections::VecDeque;
use std::io::{Cursor, ErrorKind};
use std::net::SocketAddr;
use std::time::Instant;

use prometheus::{Histogram, HistogramOpts, IntCounter};

use crate::network::socket::UdpSocketWrapper;
use crate::protocol::codec::cipher::Cipher;
use crate::protocol::disconnect::command::DisconnectByCommandReason;
use crate::protocol::frame::headers::Header;
use crate::protocol::frame::input::InFrame;
use crate::protocol::frame::output::OutFrame;
use crate::protocol::frame::ConnectionId;
use crate::protocol::others::member_id::MemberAndRoomId;
use crate::protocol::Protocol;
use crate::room::{MemberPrivateKey, RoomId, RoomMemberId};

#[derive(Debug)]
pub struct NetworkChannel {
	pub state: ConnectionStatus,
	pub current_connection_id: ConnectionId,
	pub protocol: Protocol,
	private_key: MemberPrivateKey,
	server_address: SocketAddr,
	pub socket_wrapper: UdpSocketWrapper,
	out_frames: VecDeque<OutFrame>,
	from_client: bool,
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DisconnectedReason {
	IOError(String),
	ByTimeout,
	ByCommand(DisconnectByCommandReason),
	ByRetransmitWhenMaxCount,
	ByRetransmitWhenMaxFrames,
	ByRetransmitWhenMaxWaitAck,
}

impl NetworkChannel {
	#[allow(clippy::unwrap_in_result)]
	pub fn new(
		connection_id: ConnectionId,
		from_client: bool,
		private_key: MemberPrivateKey,
		member_id: RoomMemberId,
		room_id: RoomId,
		server_address: SocketAddr,
		start_application_time: Instant,
	) -> std::io::Result<NetworkChannel> {
		let protocol = Protocol::new(
			connection_id,
			Instant::now(),
			start_application_time,
			IntCounter::new("name", "help").unwrap().local(),
			Histogram::with_opts(HistogramOpts::new("name", "help")).unwrap().local(),
		);
		let channel = UdpSocketWrapper::new()?;

		Ok(NetworkChannel {
			current_connection_id: 0,
			state: ConnectionStatus::Connecting,
			protocol,
			private_key,
			server_address,
			socket_wrapper: channel,
			out_frames: Default::default(),
			from_client,
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
		while let Some(mut frame) = self.protocol.build_next_frame(now) {
			frame.headers.add(Header::MemberAndRoomId(self.member_and_room_id));
			self.out_frames.push_front(frame);
		}

		let mut buffer = [0; 2048];
		while let Some(frame) = self.out_frames.back_mut() {
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
					let mut cursor = Cursor::new(&buffer[0..size]);
					let header = InFrame::decode_meta(&mut cursor);
					match header {
						Ok((connection_id, frame_id, headers)) => match InFrame::decode_commands(self.from_client, frame_id, cursor, Cipher::new(&self.private_key)) {
							Ok(commands) => {
								let frame = InFrame::new(connection_id, frame_id, headers, commands);
								self.on_frame_received(now, &frame);
							}
							Err(e) => {
								tracing::error!("error decode frame {:?}", e);
							}
						},
						Err(e) => {
							tracing::error!("error decode header {:?}", e);
						}
					}
				}
			}
		}
	}

	fn on_frame_received(&mut self, now: Instant, frame: &InFrame) {
		self.protocol.on_frame_received(frame, now);
	}
}
