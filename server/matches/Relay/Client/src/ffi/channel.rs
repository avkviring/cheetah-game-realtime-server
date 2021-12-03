use cheetah_matches_relay_common::protocol::frame::applications::ChannelGroupId;

use crate::ffi::execute_with_client;

#[derive(Debug, Clone)]
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
pub extern "C" fn set_channel(channel: Channel, group: ChannelGroupId) -> bool {
	execute_with_client(|client| client.set_current_channel(channel.clone(), group)).is_ok()
}
