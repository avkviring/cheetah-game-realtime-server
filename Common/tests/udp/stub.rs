use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Range, RangeInclusive};
use std::rc::Rc;

use rand::{Rng, RngCore};
use rand::rngs::OsRng;

use cheetah_relay_common::commands::hash::{UserPrivateKey, UserPublicKey};
use cheetah_relay_common::udp::channel::{Channel, Transport, TransportError};
use cheetah_relay_common::udp::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel, ApplicationCommandDescription};

#[derive(Hash, Clone, Debug, Eq, PartialEq, Copy)]
pub struct AddressStub {
	pub id: u64
}

impl AddressStub {
	pub fn new(id: u64) -> Self {
		Self {
			id
		}
	}
}

pub struct TransportStub {
	pub source_to_packet: Rc<RefCell<HashMap<AddressStub, Vec<PacketStub>>>>,
	pub channel_quality: ChannelQuality,
}


#[derive(Default, Clone)]
pub struct ChannelQuality {
	reliable_percents: Vec<(RangeInclusive<u64>, f64)>
}

impl ChannelQuality {
	pub fn add_reliable_percent(&mut self, range: RangeInclusive<u64>, transfered_percent: f64) {
		self.reliable_percents.push((range, transfered_percent));
	}
	
	pub fn allow(&self, position: u64) -> bool {
		let rule = self.reliable_percents.iter().find(|(range, percent)| {
			range.contains(&position)
		});
		let result = match rule {
			None => {
				true
			}
			Some((_, percent)) => {
				OsRng.gen_bool(*percent)
			}
		};
		result
	}
}


impl TransportStub {
	pub fn new(channel_quality: ChannelQuality) -> Box<dyn Transport<AddressStub>> {
		Box::new(TransportStub {
			source_to_packet: Default::default(),
			channel_quality,
		})
	}
}

impl Transport<AddressStub> for TransportStub {
	fn create_channel(&self, self_address: AddressStub) -> Result<Rc<RefCell<dyn Channel<AddressStub>>>, TransportError> {
		Result::Ok(Rc::new(
			RefCell::new(
				ChannelStub {
					self_address,
					source_to_packet: self.source_to_packet.clone(),
					lost_controller: self.channel_quality.clone(),
					position: Default::default(),
					
				})))
	}
}


#[derive(Debug)]
pub struct PacketStub {
	pub from: AddressStub,
	pub buffer: Vec<u8>,
}

pub struct ChannelStub {
	pub self_address: AddressStub,
	pub source_to_packet: Rc<RefCell<HashMap<AddressStub, Vec<PacketStub>>>>,
	pub lost_controller: ChannelQuality,
	pub position: u64,
}

impl Channel<AddressStub> for ChannelStub {
	fn send(&mut self, to: &AddressStub, buffer: &[u8]) -> Result<usize, TransportError> {
		self.position += 1;
		if !self.lost_controller.allow(self.position) {
			return Result::Ok(buffer.len());
		}
		let cloned = self.source_to_packet.clone();
		let packets = &mut *cloned.borrow_mut();
		let v = packets.entry(to.clone()).or_insert_with(|| Vec::default());
		v.push(PacketStub {
			from: self.self_address,
			buffer: buffer.to_vec(),
		});
		
		Result::Ok(buffer.len())
	}
	
	
	fn receive(&self, buf: &mut [u8]) -> Result<(usize, AddressStub), TransportError> {
		let cloned = self.source_to_packet.clone();
		let packets = &mut *cloned.borrow_mut();
		match packets.get_mut(&self.self_address) {
			None => { Result::Err(TransportError::NoData) }
			Some(v) => {
				match v.pop() {
					None => { Result::Err(TransportError::NoData) }
					Some(p) => {
						buf[0..p.buffer.len()].copy_from_slice(p.buffer.as_slice());
						Result::Ok((p.buffer.len(), p.from))
					}
				}
			}
		}
	}
}

pub fn create_user_private_key_stub() -> UserPrivateKey {
	let mut result = [0; 32];
	OsRng.fill_bytes(&mut result);
	result
}


pub fn create_user_public_key_stub() -> UserPublicKey {
	let mut result = [0; 4];
	OsRng.fill_bytes(&mut result);
	result
}

pub fn new_ping_command(ping: String) -> ApplicationCommandDescription {
	ApplicationCommandDescription::new(ApplicationCommandChannel::Unordered, ApplicationCommand::TestSimple(ping))
}

