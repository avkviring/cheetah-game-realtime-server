use cheetah_common::protocol::frame::applications::ChannelGroup;

use crate::clients::registry::ClientId;
use crate::ffi::execute_with_client;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum Channel {
	ReliableUnordered,
	UnreliableUnordered,
	ReliableOrdered,
	UnreliableOrdered,
	ReliableSequence,
}

#[no_mangle]
pub extern "C" fn set_channel(client_id: ClientId, channel: Channel, group: u8) -> u8 {
	execute_with_client(client_id, |client| {
		client.set_current_channel(channel, ChannelGroup(group));
		Ok(())
	})
}
