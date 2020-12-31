use std::collections::VecDeque;
use std::io::{Cursor, ErrorKind};
use std::net::SocketAddr;
use std::time::Instant;

use crate::network::channel::NetworkChannel;
use crate::protocol::codec::cipher::Cipher;
use crate::protocol::frame::Frame;
use crate::protocol::others::user_id::{UserAndRoomId, UserIdFrameBuilder};
use crate::protocol::relay::RelayProtocol;
use crate::room::{RoomId, UserId, UserPrivateKey};

#[derive(Debug)]
pub struct NetworkClient {
	pub state: ConnectionStatus,
	pub protocol: RelayProtocol,
	private_key: UserPrivateKey,
	server_address: SocketAddr,
	channel: NetworkChannel,
	out_frames: VecDeque<Frame>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(C)]
pub enum ConnectionStatus {
	Connecting,
	///
	/// Соединение установлено
	///
	Connected,
	///
	/// Соединение закрыто
	///
	Disconnected,
}

impl NetworkClient {
	pub fn new(
		private_key: UserPrivateKey,
		user_id: UserId,
		room_id: RoomId,
		server_address: SocketAddr,
		start_frame_id: u64,
	) -> Result<NetworkClient, ()> {
		let mut protocol = RelayProtocol::new(&Instant::now());
		protocol.next_frame_id = start_frame_id;

		protocol.add_frame_builder(Box::new(UserIdFrameBuilder(UserAndRoomId { user_id, room_id })));
		let channel = NetworkChannel::new()?;

		Result::Ok(NetworkClient {
			state: ConnectionStatus::Connecting,
			protocol,
			private_key,
			server_address,
			channel,
			out_frames: Default::default(),
		})
	}

	pub fn cycle(&mut self, now: &Instant) {
		if self.state == ConnectionStatus::Disconnected {
			return;
		}

		self.protocol.cycle(now);
		self.do_read(now);
		self.do_write(now);

		if self.protocol.connected(now) {
			self.state = ConnectionStatus::Connected
		}

		if self.protocol.disconnected(now) {
			self.state = ConnectionStatus::Disconnected
		}
	}

	fn do_write(&mut self, now: &Instant) {
		while let Some(frame) = self.protocol.build_next_frame(&now) {
			self.out_frames.push_front(frame);
		}

		let mut buffer = [0; 2048];
		while let Some(frame) = self.out_frames.back() {
			let frame_buffer_size = frame.encode(&mut Cipher::new(&self.private_key), &mut buffer);
			match self.channel.send_to(&buffer[0..frame_buffer_size], self.server_address) {
				Ok(size) => {
					if size != frame_buffer_size {
						log::error!("error send frame size mismatch send {:?}, frame {:?}", size, frame_buffer_size);
					} else {
						self.out_frames.pop_back();
					}
				}
				Err(e) => match e.kind() {
					ErrorKind::WouldBlock => {}
					_ => {
						log::error!("error send {:?}", e);
						self.state = ConnectionStatus::Disconnected;
					}
				},
			}
		}
	}

	fn do_read(&mut self, now: &Instant) {
		let mut buffer = [0; 2048];
		loop {
			match self.channel.recv(&mut buffer) {
				Err(e) => {
					match e.kind() {
						ErrorKind::WouldBlock => {}
						_ => {
							log::error!("error receive {:?}", e);
							self.state = ConnectionStatus::Disconnected;
						}
					}
					break;
				}
				Ok(size) => {
					let mut cursor = Cursor::new(&buffer[0..size]);
					let header = Frame::decode_headers(&mut cursor);
					match header {
						Ok((header, additional_headers)) => {
							let frame = Frame::decode_frame(cursor, Cipher::new(&self.private_key), header, additional_headers);
							match frame {
								Ok(frame) => {
									self.on_frame_received(now, frame);
								}
								Err(e) => {
									log::error!("error decode frame {:?}", e)
								}
							}
						}
						Err(e) => {
							log::error!("skip protocol by header {:?}", e)
						}
					}
				}
			}
		}
	}

	fn on_frame_received(&mut self, now: &Instant, frame: Frame) {
		self.protocol.on_frame_received(frame, now);
	}
}
