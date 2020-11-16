use cheetah_relay_common::protocol::frame::applications::ChannelGroupId;

use crate::ffi::execute_with_client;

#[derive(Debug)]
#[repr(C)]
pub enum Channel {
	ReliableUnordered,
	UnreliableUnordered,
	ReliableOrderedByObject,
	UnreliableOrderedByObject,
	ReliableSequenceByObject,
	ReliableOrderedByGroup,
	UnreliableOrderedByGroup,
	ReliableSequenceByGroup,
	
}

#[no_mangle]
#[allow(unused_must_use)]
pub extern fn set_current_channel(channel: Channel, group: ChannelGroupId) {
	execute_with_client(|client| {
		client.set_current_channel(channel, group);
	});
}
