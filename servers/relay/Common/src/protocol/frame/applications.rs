use serde::{Deserialize, Serialize};

use crate::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand, S2CCommandWithMeta};
use crate::room::object::GameObjectId;

pub type ChannelGroupId = u16;
pub type ChannelSequence = u32;

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ApplicationCommand {
	TestSimple(String),
	TestObject(GameObjectId, String),
	S2CCommandWithMeta(S2CCommandWithMeta),
	C2SCommandWithMeta(C2SCommandWithMeta),
}

///
/// Тип канала для отправки
///
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ApplicationCommandChannelType {
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
	ReliableOrderedByGroup(ChannelGroupId),
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
	UnreliableOrderedByGroup(ChannelGroupId),
	///
	/// Выполняем команды строго по-порядку по объекту
	///
	ReliableSequenceByObject,
	///
	/// Выполняем команды строго по-порядку по группе
	///
	ReliableSequenceByGroup(ChannelGroupId),
}

///
/// Канал для отправки, отличается от [ApplicationCommandChannelType] полным набором данных для канала
///
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
	ReliableOrderedByGroup(ChannelGroupId),
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
	UnreliableOrderedByGroup(ChannelGroupId),
	///
	/// Выполняем команды строго по-порядку по объекту
	///
	ReliableSequenceByObject(ChannelSequence),
	///
	/// Выполняем команды строго по-порядку по группе
	///
	ReliableSequenceByGroup(ChannelGroupId, ChannelSequence),
}

impl From<&ApplicationCommandChannel> for ApplicationCommandChannelType {
	fn from(channel: &ApplicationCommandChannel) -> Self {
		match channel {
			ApplicationCommandChannel::ReliableUnordered => ApplicationCommandChannelType::ReliableUnordered,
			ApplicationCommandChannel::ReliableOrderedByObject => ApplicationCommandChannelType::ReliableOrderedByObject,
			ApplicationCommandChannel::ReliableOrderedByGroup(channel) => ApplicationCommandChannelType::ReliableOrderedByGroup(*channel),
			ApplicationCommandChannel::UnreliableUnordered => ApplicationCommandChannelType::UnreliableUnordered,
			ApplicationCommandChannel::UnreliableOrderedByObject => ApplicationCommandChannelType::UnreliableOrderedByObject,
			ApplicationCommandChannel::UnreliableOrderedByGroup(channel) => ApplicationCommandChannelType::UnreliableOrderedByGroup(*channel),
			ApplicationCommandChannel::ReliableSequenceByObject(_) => ApplicationCommandChannelType::ReliableSequenceByObject,
			ApplicationCommandChannel::ReliableSequenceByGroup(channel, _) => ApplicationCommandChannelType::ReliableSequenceByGroup(*channel),
		}
	}
}

impl ApplicationCommand {
	pub fn get_object_id(&self) -> Option<&GameObjectId> {
		match &self {
			ApplicationCommand::TestSimple(_) => Option::None,
			ApplicationCommand::TestObject(object_id, _) => Option::Some(object_id),
			ApplicationCommand::S2CCommandWithMeta(command_with_meta) => match &command_with_meta.command {
				S2CCommand::Create(c) => Option::Some(&c.object_id),
				S2CCommand::SetLong(c) => Option::Some(&c.object_id),
				S2CCommand::SetFloat(c) => Option::Some(&c.object_id),
				S2CCommand::SetStruct(c) => Option::Some(&c.object_id),
				S2CCommand::Event(c) => Option::Some(&c.object_id),
				S2CCommand::Delete(c) => Option::Some(&c.object_id),
				S2CCommand::Created(c) => Option::Some(&c.object_id),
			},
			ApplicationCommand::C2SCommandWithMeta(command_with_meta) => match &command_with_meta.command {
				C2SCommand::Create(c) => Option::Some(&c.object_id),
				C2SCommand::SetLong(c) => Option::Some(&c.object_id),
				C2SCommand::IncrementLongValue(c) => Option::Some(&c.object_id),
				C2SCommand::SetFloat(c) => Option::Some(&c.object_id),
				C2SCommand::IncrementFloatCounter(c) => Option::Some(&c.object_id),
				C2SCommand::SetStruct(c) => Option::Some(&c.object_id),
				C2SCommand::Event(c) => Option::Some(&c.object_id),
				C2SCommand::Delete(c) => Option::Some(&c.object_id),
				C2SCommand::AttachToRoom => Option::None,
				C2SCommand::CompareAndSetLongValue(c) => Option::Some(&c.object_id),
				C2SCommand::Created(c) => Option::Some(&c.object_id),
			},
		}
	}
}
