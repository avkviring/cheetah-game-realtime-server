use hash32_derive::Hash32;

use crate::commands::c2s::C2SCommand;
use crate::commands::s2c::S2CCommandWithCreator;
use crate::protocol::frame::channel::Channel;
use crate::room::object::GameObjectId;

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, Default, Hash32)]
#[repr(C)]
pub struct ChannelGroup(pub u8);

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, Default)]
#[repr(C)]
pub struct ChannelSequence(pub u32);

#[derive(Debug, PartialEq, Clone)]
pub struct CommandWithChannel {
	pub channel: Channel,
	pub both_direction_command: BothDirectionCommand,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BothDirectionCommand {
	S2CWithCreator(S2CCommandWithCreator),
	C2S(C2SCommand),
}

impl ChannelSequence {
	pub const FIRST: ChannelSequence = ChannelSequence(0);

	pub fn next(&self) -> ChannelSequence {
		ChannelSequence(self.0 + 1)
	}
}

impl BothDirectionCommand {
	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match &self {
			BothDirectionCommand::S2CWithCreator(command_with_meta) => command_with_meta.command.get_object_id(),
			BothDirectionCommand::C2S(command) => command.get_object_id(),
		}
	}
}
