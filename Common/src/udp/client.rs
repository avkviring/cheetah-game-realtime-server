use std::hash::Hash;
use std::io::Cursor;
use std::time::Instant;

use crate::commands::hash::{UserPrivateKey, UserPublicKey};
use crate::udp::channel::Channel;
use crate::udp::client::ClientState::CONNECTED;
use crate::udp::protocol::codec::cipher::Cipher;
use crate::udp::protocol::frame::Frame;
use crate::udp::protocol::others::public_key::UserPublicKeyFrameBuilder;
use crate::udp::protocol::relay::RelayProtocol;

pub struct UdpClient<PeerAddres> {
	pub private_key: UserPrivateKey,
	pub public_key: UserPublicKey,
	pub channel: Box<dyn Channel<PeerAddres>>,
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
			   channel: Box<dyn Channel<PeerAddress>>,
			   server_address: PeerAddress) -> UdpClient<PeerAddress> {
		let mut protocol = RelayProtocol::new();
		protocol.add_frame_builder(Box::new(UserPublicKeyFrameBuilder(public_key)));
		UdpClient {
			private_key,
			public_key,
			channel,
			state: ClientState::DISCONNECTED,
			server_address,
			protocol,
		}
	}
	
	pub fn cycle(&mut self, now:Instant) {
		loop {
			// read
			match self.channel.try_recv() {
				None => { break; }
				Some((server_address, data)) => {
					let mut cursor = Cursor::new(data.as_slice());
					let header = Frame::decode_headers(&mut cursor);
					match header {
						Ok((header, additional_headers)) => {
							let frame = Frame::decode_frame(cursor, Cipher::new(&self.private_key), header, additional_headers);
							match frame {
								Ok(frame) => {
									self.state = CONNECTED;
									println!("client recv protocol {:?}", frame)
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
		
		//write
		let frame = self.protocol.build_next_frame(&now);
		match frame {
			None => {}
			Some(mut frame) => {
				let (buffer, unsended_commands) = frame.encode(&mut Cipher::new(&self.private_key));
				self.channel.send(&self.server_address, buffer);
				self.protocol.out_commands_collector.add_unsent_commands(unsended_commands);
			}
		}
	}
}

