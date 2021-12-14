use std::collections::VecDeque;

use crate::commands::c2s::C2SCommand;
use crate::commands::s2c::{S2CCommand, S2CCommandWithCreator};
use crate::protocol::frame::channel::CommandChannel;
use crate::room::object::GameObjectId;

pub type ChannelGroup = u16;
pub type ChannelSequence = u32;

///
/// Прикладные команды
///
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ApplicationCommands {
	///
	/// С гарантией доставки
	///
	pub reliable: VecDeque<CommandWithChannel>,

	///
	/// Без гарантии доставки
	///
	pub unreliable: VecDeque<CommandWithChannel>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CommandWithChannel {
	pub channel: CommandChannel,
	pub command: BothDirectionCommand,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BothDirectionCommand {
	TestSimple(String),
	TestObject(GameObjectId, String),
	S2CCommandWithCreator(S2CCommandWithCreator),
	C2SCommand(C2SCommand),
}

impl BothDirectionCommand {
	pub fn get_object_id(&self) -> Option<&GameObjectId> {
		match &self {
			BothDirectionCommand::TestSimple(_) => Option::None,
			BothDirectionCommand::TestObject(object_id, _) => Option::Some(object_id),
			BothDirectionCommand::S2CCommandWithCreator(command_with_meta) => match &command_with_meta.command {
				S2CCommand::Create(c) => Option::Some(&c.object_id),
				S2CCommand::SetLong(c) => Option::Some(&c.object_id),
				S2CCommand::SetFloat(c) => Option::Some(&c.object_id),
				S2CCommand::SetStruct(c) => Option::Some(&c.object_id),
				S2CCommand::Event(c) => Option::Some(&c.object_id),
				S2CCommand::Delete(c) => Option::Some(&c.object_id),
				S2CCommand::Created(c) => Option::Some(&c.object_id),
			},
			BothDirectionCommand::C2SCommand(command) => match command {
				C2SCommand::Create(c) => Option::Some(&c.object_id),
				C2SCommand::SetLong(c) => Option::Some(&c.object_id),
				C2SCommand::IncrementLongValue(c) => Option::Some(&c.object_id),
				C2SCommand::SetFloat(c) => Option::Some(&c.object_id),
				C2SCommand::IncrementFloatCounter(c) => Option::Some(&c.object_id),
				C2SCommand::SetStruct(c) => Option::Some(&c.object_id),
				C2SCommand::Event(c) => Option::Some(&c.object_id),
				C2SCommand::Delete(c) => Option::Some(&c.object_id),
				C2SCommand::AttachToRoom => Option::None,
				C2SCommand::DetachFromRoom => Option::None,
				C2SCommand::CompareAndSetLongValue(c) => Option::Some(&c.object_id),
				C2SCommand::Created(c) => Option::Some(&c.object_id),
				C2SCommand::TargetEvent(c) => Option::Some(&c.event.object_id),
			},
		}
	}
}
