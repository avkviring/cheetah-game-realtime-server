use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Range, RangeInclusive};
use std::rc::Rc;

use rand::Rng;
use rand::rngs::OsRng;

use cheetah_relay_common::udp::channel::{Channel, Transport};

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
	rules: Vec<(RangeInclusive<u64>, f64)>
}

impl ChannelQuality {
	pub fn add_reliable_percent(&mut self, range: RangeInclusive<u64>, transfered_percent: f64) {
		self.rules.push((range, transfered_percent));
	}
	
	pub fn allow(&self, position: u64) -> bool {
		
		let rule = self.rules.iter().find(|(range, percent)| {
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
	fn create_channel(&self, self_address: AddressStub) -> Rc<RefCell<dyn Channel<AddressStub>>> {
		Rc::new(
			RefCell::new(
				ChannelStub {
					self_address,
					source_to_packet: self.source_to_packet.clone(),
					lost_controller: self.channel_quality.clone(),
					position: Default::default(),
					
				}))
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
	fn send(&mut self, to: &AddressStub, buffer: Vec<u8>) {
		self.position += 1;
		if !self.lost_controller.allow(self.position) {
			return;
		}
		let cloned = self.source_to_packet.clone();
		let packets = &mut *cloned.borrow_mut();
		let v = packets.entry(to.clone()).or_insert_with(|| Vec::default());
		v.push(PacketStub {
			from: self.self_address,
			buffer,
		});
	}
	
	fn try_recv(&self) -> Option<(AddressStub, Vec<u8>)> {
		let cloned = self.source_to_packet.clone();
		let packets = &mut *cloned.borrow_mut();
		match packets.get_mut(&self.self_address) {
			None => { Option::None }
			Some(v) => { v.pop().map(|p| (p.from, p.buffer)) }
		}
	}
}