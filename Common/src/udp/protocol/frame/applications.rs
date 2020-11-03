use serde::{Deserialize, Serialize};

use crate::commands::command::load::LoadGameObjectCommand;
use crate::room::object::ClientGameObjectId;

///
/// Прикладные команды
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct ApplicationCommands {
	///
	/// С гарантией доставки
	///
	pub reliability: Vec<ApplicationCommandDescription>,
	
	///
	/// Без гарантии доставки
	///
	pub unreliability: Vec<ApplicationCommandDescription>,
	
}

impl ApplicationCommands {
	pub fn add(&mut self, command: &Self) {
		self.reliability.extend_from_slice(&command.reliability);
		self.unreliability.extend_from_slice(&command.unreliability);
	}
	
	pub fn clear(&mut self) {
		self.reliability.clear();
		self.unreliability.clear();
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
	TestObject(ClientGameObjectId, String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ApplicationCommandChannel {
	///
	/// Выполняем команды без учета порядка
	///
	Unordered,
	///
	/// Отбрасываем команды из прошлого по объекту
	///
	OrderedByObject,
	///
	/// Отбрасываем команды из прошлого по группе
	///
	OrderedByGroup(ChannelId),
	///
	/// Выполняем команды строго по-порядку по объекту
	///
	SequenceByObject(ChannelSequence),
	///
	/// Выполняем команды строго по-порядку по группе
	///
	SequenceByGroup(ChannelId, ChannelSequence),
}

pub type ChannelId = u16;
pub type ChannelSequence = u32;


impl ApplicationCommand {
	pub fn get_object_id(&self) -> Option<&ClientGameObjectId> {
		match self {
			ApplicationCommand::TestSimple(_) => { Option::None }
			ApplicationCommand::TestObject(object_id, _) => {
				Option::Some(object_id)
			}
		}
	}
}

