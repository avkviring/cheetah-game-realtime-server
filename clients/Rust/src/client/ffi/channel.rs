use cheetah_relay_common::protocol::frame::applications::ApplicationCommandChannel;

use crate::client::ffi::{ChannelFFI, Command};

impl Command {
	pub fn to_channel(&self) -> ApplicationCommandChannel {
		match self.channel {
			ChannelFFI::None => { panic!("channelFFI not set") }
			ChannelFFI::ReliableUnordered => {
				ApplicationCommandChannel::ReliableUnordered
			}
			ChannelFFI::ReliableOrderedByObject => {
				ApplicationCommandChannel::ReliableOrderedByObject
			}
			ChannelFFI::ReliableOrderedByGroup => {
				ApplicationCommandChannel::ReliableOrderedByGroup(self.channel_group_id)
			}
			ChannelFFI::UnreliableUnordered => {
				ApplicationCommandChannel::UnreliableUnordered
			}
			ChannelFFI::UnreliableOrderedByObject => {
				ApplicationCommandChannel::UnreliableOrderedByObject
			}
			ChannelFFI::UnreliableOrderedByGroup => {
				ApplicationCommandChannel::UnreliableOrderedByGroup(self.channel_group_id)
			}
			ChannelFFI::ReliableSequenceByObject => {
				ApplicationCommandChannel::ReliableSequenceByObject(self.channel_sequence)
			}
			ChannelFFI::ReliableSequenceByGroup => {
				ApplicationCommandChannel::ReliableSequenceByGroup(self.channel_group_id, self.channel_sequence)
			}
		}
	}
}