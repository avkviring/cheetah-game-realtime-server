use serde::{Deserialize, Serialize};
use crate::protocol::frame::applications::{ChannelGroup, ChannelSequence};

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
	ReliableOrderedByGroup(ChannelGroup),
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
	UnreliableOrderedByGroup(ChannelGroup),
	///
	/// Выполняем команды строго по-порядку по объекту
	///
	ReliableSequenceByObject,
	///
	/// Выполняем команды строго по-порядку по группе
	///
	ReliableSequenceByGroup(ChannelGroup),
}

///
/// Канал для отправки, отличается от [ApplicationCommandChannelType] полным набором данных для канала
///
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum CommandChannel {
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
	ReliableOrderedByGroup(ChannelGroup),
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
	UnreliableOrderedByGroup(ChannelGroup),
	///
	/// Выполняем команды строго по-порядку по объекту
	///
	ReliableSequenceByObject(ChannelSequence),
	///
	/// Выполняем команды строго по-порядку по группе
	///
	ReliableSequenceByGroup(ChannelGroup, ChannelSequence),
}

impl From<&CommandChannel> for ApplicationCommandChannelType {
	fn from(channel: &CommandChannel) -> Self {
		match channel {
			CommandChannel::ReliableUnordered => ApplicationCommandChannelType::ReliableUnordered,
			CommandChannel::ReliableOrderedByObject => ApplicationCommandChannelType::ReliableOrderedByObject,
			CommandChannel::ReliableOrderedByGroup(channel) => {
				ApplicationCommandChannelType::ReliableOrderedByGroup(*channel)
			}
			CommandChannel::UnreliableUnordered => ApplicationCommandChannelType::UnreliableUnordered,
			CommandChannel::UnreliableOrderedByObject => ApplicationCommandChannelType::UnreliableOrderedByObject,
			CommandChannel::UnreliableOrderedByGroup(channel) => {
				ApplicationCommandChannelType::UnreliableOrderedByGroup(*channel)
			}
			CommandChannel::ReliableSequenceByObject(_) => ApplicationCommandChannelType::ReliableSequenceByObject,
			CommandChannel::ReliableSequenceByGroup(channel, _) => {
				ApplicationCommandChannelType::ReliableSequenceByGroup(*channel)
			}
		}
	}
}

impl CommandChannel {
	pub fn get_channel_group_id(&self) -> Option<ChannelGroup> {
		match self {
			CommandChannel::ReliableUnordered => None,
			CommandChannel::ReliableOrderedByObject => None,
			CommandChannel::ReliableOrderedByGroup(group) => Some(*group),
			CommandChannel::UnreliableUnordered => None,
			CommandChannel::UnreliableOrderedByObject => None,
			CommandChannel::UnreliableOrderedByGroup(group) => Some(*group),
			CommandChannel::ReliableSequenceByObject(_) => None,
			CommandChannel::ReliableSequenceByGroup(group, _) => Some(*group),
		}
	}
}
