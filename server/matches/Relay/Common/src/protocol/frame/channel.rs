use crate::protocol::frame::applications::{ChannelGroup, ChannelSequence};

///
/// Тип канала для отправки
///
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Clone)]
pub enum Channel {
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

impl From<&Channel> for ApplicationCommandChannelType {
	fn from(channel: &Channel) -> Self {
		match channel {
			Channel::ReliableUnordered => ApplicationCommandChannelType::ReliableUnordered,
			Channel::ReliableOrderedByObject => ApplicationCommandChannelType::ReliableOrderedByObject,
			Channel::ReliableOrderedByGroup(channel) => ApplicationCommandChannelType::ReliableOrderedByGroup(*channel),
			Channel::UnreliableUnordered => ApplicationCommandChannelType::UnreliableUnordered,
			Channel::UnreliableOrderedByObject => ApplicationCommandChannelType::UnreliableOrderedByObject,
			Channel::UnreliableOrderedByGroup(channel) => {
				ApplicationCommandChannelType::UnreliableOrderedByGroup(*channel)
			}
			Channel::ReliableSequenceByObject(_) => ApplicationCommandChannelType::ReliableSequenceByObject,
			Channel::ReliableSequenceByGroup(channel, _) => {
				ApplicationCommandChannelType::ReliableSequenceByGroup(*channel)
			}
		}
	}
}

impl Channel {
	pub fn get_channel_group_id(&self) -> Option<ChannelGroup> {
		match self {
			Channel::ReliableUnordered => None,
			Channel::ReliableOrderedByObject => None,
			Channel::ReliableOrderedByGroup(group) => Some(*group),
			Channel::UnreliableUnordered => None,
			Channel::UnreliableOrderedByObject => None,
			Channel::UnreliableOrderedByGroup(group) => Some(*group),
			Channel::ReliableSequenceByObject(_) => None,
			Channel::ReliableSequenceByGroup(group, _) => Some(*group),
		}
	}
}
