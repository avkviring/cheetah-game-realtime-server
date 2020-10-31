use std::cell::RefCell;
use std::hash::Hash;
use std::io::Cursor;
use std::rc::Rc;
use std::time::Instant;

use crate::commands::hash::{UserPrivateKey, UserPublicKey};
use crate::udp::channel::Channel;
use crate::udp::client::ClientState::CONNECTED;
use crate::udp::protocol::codec::cipher::Cipher;
use crate::udp::protocol::frame::Frame;
use crate::udp::protocol::others::hello::HelloSender;
use crate::udp::protocol::others::public_key::UserPublicKeyFrameBuilder;
use crate::udp::protocol::relay::RelayProtocol;

pub struct UdpClient<PeerAddres> {
	pub private_key: UserPrivateKey,
	pub public_key: UserPublicKey,
	pub channel: Rc<RefCell<dyn Channel<PeerAddres>>>,
	pub state: ClientState,
	pub server_address: PeerAddres,
	pub protocol: RelayProtocol,
}

#[derive(Debug, PartialEq)]
pub enum ClientState {
	DISCONNECTED,
	CONNECTED,
}


impl<PeerAddress: Hash> UdpClient<PeerAddress> {
	pub fn new(private_key: UserPrivateKey,
			   public_key: UserPublicKey,
			   channel: Rc<RefCell<dyn Channel<PeerAddress>>>,
			   server_address: PeerAddress) -> UdpClient<PeerAddress> {
		let mut protocol = RelayProtocol::new();
		protocol.add_frame_builder(Box::new(UserPublicKeyFrameBuilder(public_key)));
		protocol.add_frame_builder(Box::new(HelloSender::default()));
		UdpClient {
			private_key,
			public_key,
			channel,
			state: ClientState::DISCONNECTED,
			server_address,
			protocol,
		}
	}
	
	pub fn cycle(&mut self, now: &Instant) {
		self.protocol.cycle(now);
		self.do_read(&now);
		self.do_write(&now)
	}
	
	fn do_write(&mut self, now: &&Instant) {
		let frame = self.protocol.build_next_frame(&now);
		match frame {
			None => {}
			Some(mut frame) => {
				let mut buffer = [0; 2048];
				let (unsended_commands, size) = frame.encode(&mut Cipher::new(&self.private_key), &mut buffer);
				let channel = self.channel.clone();
				channel.borrow_mut().send(&self.server_address, &buffer[0..size]).ok().expect("write fail");
				self.protocol.out_commands_collector.add_unsent_commands(unsended_commands);
			}
		}
	}
	
	fn do_read(&mut self, now: &&Instant) {
		let mut buffer = [0; 2048];
		loop {
			let channel = self.channel.clone();
			let channel = channel.borrow();
			match channel.receive(&mut buffer) {
				Err(_) => { break; }
				Ok((size, _)) => {
					let mut cursor = Cursor::new(&buffer[0..size]);
					let header = Frame::decode_headers(&mut cursor);
					match header {
						Ok((header, additional_headers)) => {
							let frame = Frame::decode_frame(cursor, Cipher::new(&self.private_key), header, additional_headers);
							match frame {
								Ok(frame) => {
									self.protocol.on_frame_received(frame, &now);
									self.state = CONNECTED;
								}
								Err(e) => {
									log::error!("recv protocol {:?}", e)
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
}

