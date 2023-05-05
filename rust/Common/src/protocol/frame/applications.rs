use crate::commands::c2s::C2SCommand;
use crate::commands::s2c::S2CCommandWithCreator;
use crate::protocol::frame::channel::ReliabilityGuaranteesChannel;
use crate::room::object::GameObjectId;

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, Default)]
#[repr(C)]
pub struct ChannelGroup(pub u8);

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, Default)]
#[repr(C)]
pub struct ChannelSequence(pub u32);

#[derive(Debug, PartialEq, Clone)]
pub struct CommandWithReliabilityGuarantees {
	pub reliability_guarantees: ReliabilityGuaranteesChannel,
	pub commands: BothDirectionCommand,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum BothDirectionCommand {
	S2CWithCreator(S2CCommandWithCreator),
	C2S(C2SCommand),
}

impl ChannelSequence {
	pub const FIRST: ChannelSequence = ChannelSequence(0);

	#[must_use]
	pub fn next(&self) -> Self {
		ChannelSequence(self.0 + 1)
	}
}

impl BothDirectionCommand {
	#[must_use]
	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match &self {
			BothDirectionCommand::S2CWithCreator(command_with_meta) => command_with_meta.command.get_object_id(),
			BothDirectionCommand::C2S(command) => command.get_object_id(),
		}
	}
}
