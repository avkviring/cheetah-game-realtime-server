use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use cheetah_relay_common::udp::channel::{Transport, Channel};


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
}

impl TransportStub {
	pub fn new() -> Box<dyn Transport<AddressStub>> {
		Box::new(TransportStub {
			source_to_packet: Default::default(),
		})
	}
}

impl Transport<AddressStub> for TransportStub {
	fn create_channel(&self, self_address: AddressStub) -> Box<dyn Channel<AddressStub>> {
		Box::new(ChannelStub {
			self_address,
			source_to_packet: self.source_to_packet.clone(),
		})
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
}

impl Channel<AddressStub> for ChannelStub {
	fn send(&self, to: &AddressStub, buffer: Vec<u8>) {
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