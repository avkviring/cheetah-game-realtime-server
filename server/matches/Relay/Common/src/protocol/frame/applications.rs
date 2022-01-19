use crate::commands::c2s::C2SCommand;
use crate::commands::s2c::{S2CCommand, S2CCommandWithCreator};
use crate::protocol::frame::channel::Channel;
use crate::room::object::GameObjectId;

pub type ChannelGroup = u16;
pub type ChannelSequence = u32;

#[derive(Debug,  PartialEq, Clone)]
pub struct CommandWithChannel {
	pub channel: Channel,
	pub command: BothDirectionCommand,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BothDirectionCommand {
	S2CWithCreator(S2CCommandWithCreator),
	C2S(C2SCommand),
}

impl BothDirectionCommand {
	pub fn get_object_id(&self) -> Option<&GameObjectId> {
		match &self {
			BothDirectionCommand::S2CWithCreator(command_with_meta) => match &command_with_meta.command {
				S2CCommand::Create(c) => Option::Some(&c.object_id),
				S2CCommand::SetLong(c) => Option::Some(&c.object_id),
				S2CCommand::SetDouble(c) => Option::Some(&c.object_id),
				S2CCommand::SetStructure(c) => Option::Some(&c.object_id),
				S2CCommand::Event(c) => Option::Some(&c.object_id),
				S2CCommand::Delete(c) => Option::Some(&c.object_id),
				S2CCommand::Created(c) => Option::Some(&c.object_id),
			},
			BothDirectionCommand::C2S(command) => match command {
				C2SCommand::Create(c) => Option::Some(&c.object_id),
				C2SCommand::SetLong(c) => Option::Some(&c.object_id),
				C2SCommand::IncrementLongValue(c) => Option::Some(&c.object_id),
				C2SCommand::SetDouble(c) => Option::Some(&c.object_id),
				C2SCommand::IncrementDouble(c) => Option::Some(&c.object_id),
				C2SCommand::SetStructure(c) => Option::Some(&c.object_id),
				C2SCommand::Event(c) => Option::Some(&c.object_id),
				C2SCommand::Delete(c) => Option::Some(&c.object_id),
				C2SCommand::AttachToRoom => Option::None,
				C2SCommand::DetachFromRoom => Option::None,
				C2SCommand::CompareAndSetLong(c) => Option::Some(&c.object_id),
				C2SCommand::Created(c) => Option::Some(&c.object_id),
				C2SCommand::TargetEvent(c) => Option::Some(&c.event.object_id),
			},
		}
	}
}
