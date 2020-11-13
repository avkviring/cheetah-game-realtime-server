use serde::{Deserialize, Serialize};

use crate::commands::command::{C2SCommandUnion, C2SCommandWithMeta, S2CCommandUnion, S2CCommandWithMeta};
use crate::room::object::GameObjectId;

///
/// Прикладные команды
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct ApplicationCommands {
	///
	/// С гарантией доставки
	///
	pub reliable: Vec<ApplicationCommandDescription>,
	
	///
	/// Без гарантии доставки
	///
	pub unreliable: Vec<ApplicationCommandDescription>,
	
}

impl ApplicationCommands {
	pub fn add_first(&mut self, command: &Self) {
		self.reliable.extend_from_slice(&command.reliable);
		self.unreliable.extend_from_slice(&command.unreliable);
	}
	
	pub fn clear(&mut self) {
		self.reliable.clear();
		self.unreliable.clear();
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ApplicationCommandDescription {
	pub channel: ApplicationCommandChannel,
	pub command: ApplicationCommand,
}


impl ApplicationCommandDescription {
	pub fn new(channel: ApplicationCommandChannel, command: ApplicationCommand) -> Self {
		Self {
			channel,
			command,
		}
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ApplicationCommand {
	TestSimple(String),
	TestObject(GameObjectId, String),
	S2CCommandWithMeta(S2CCommandWithMeta),
	C2SCommandWithMeta(C2SCommandWithMeta),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ApplicationCommandChannel {
	///
	/// Выполняем команды без учета порядка
	///
	ReliableUnordered,
	///
	/// Отбрасываем команды из прошлого по объекту
	///
	ReliableOrderedByObject,
	///
	/// Отбрасываем команды из прошлого по группе
	///
	ReliableOrderedByGroup(GroupId),
	///
	/// Выполняем команды без учета порядка
	///
	UnreliableUnordered,
	///
	/// Отбрасываем команды из прошлого по объекту
	///
	UnreliableOrderedByObject,
	///
	/// Отбрасываем команды из прошлого по группе
	///
	UnreliableOrderedByGroup(GroupId),
	///
	/// Выполняем команды строго по-порядку по объекту
	///
	ReliableSequenceByObject(ChannelSequence),
	///
	/// Выполняем команды строго по-порядку по группе
	///
	ReliableSequenceByGroup(GroupId, ChannelSequence),
}

pub type GroupId = u16;
pub type ChannelSequence = u32;


impl ApplicationCommand {
	pub fn get_object_id(&self) -> Option<&GameObjectId> {
		match &self {
			ApplicationCommand::TestSimple(_) => { Option::None }
			ApplicationCommand::TestObject(object_id, _) => {
				Option::Some(object_id)
			}
			ApplicationCommand::S2CCommandWithMeta(command_with_meta) => {
				match &command_with_meta.command {
					S2CCommandUnion::Create(c) => {
						Option::Some(&c.object_id)
					}
					S2CCommandUnion::SetLong(c) => {
						Option::Some(&c.object_id)
					}
					S2CCommandUnion::SetFloat64(c) => {
						Option::Some(&c.object_id)
					}
					S2CCommandUnion::SetStruct(c) => {
						Option::Some(&c.object_id)
					}
					S2CCommandUnion::Event(c) => {
						Option::Some(&c.object_id)
					}
					S2CCommandUnion::Delete(c) => {
						Option::Some(&c.object_id)
					}
				}
			}
			ApplicationCommand::C2SCommandWithMeta(command_with_meta) => {
				match &command_with_meta.command {
					C2SCommandUnion::Create(c) => {
						Option::Some(&c.object_id)
					}
					C2SCommandUnion::SetLongCounter(c) => {
						Option::Some(&c.object_id)
					}
					C2SCommandUnion::IncrementLongCounter(c) => {
						Option::Some(&c.object_id)
					}
					C2SCommandUnion::SetFloatCounter(c) => {
						Option::Some(&c.object_id)
					}
					C2SCommandUnion::IncrementFloatCounter(c) => {
						Option::Some(&c.object_id)
					}
					C2SCommandUnion::Structure(c) => {
						Option::Some(&c.object_id)
					}
					C2SCommandUnion::Event(c) => {
						Option::Some(&c.object_id)
					}
					C2SCommandUnion::Delete(c) => {
						Option::Some(&c.object_id)
					}
					C2SCommandUnion::Test(_) => {
						Option::None
					}
					C2SCommandUnion::LoadRoom => {
						Option::None
					}
				}
			}
		}
	}
}

