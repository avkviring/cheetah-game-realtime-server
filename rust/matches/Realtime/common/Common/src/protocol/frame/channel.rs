use crate::protocol::frame::applications::{ChannelGroup, ChannelSequence};

///
/// Тип канала для отправки
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChannelType {
	///
	/// Выполняем команды без учета порядка
	///
	ReliableUnordered,
	///
	/// Выполняем команды без учета порядка
	///
	UnreliableUnordered,
	///
	/// Отбрасываем команды из прошлого по группе
	///
	ReliableOrdered(ChannelGroup),
	///
	/// Отбрасываем команды из прошлого по группе
	///
	UnreliableOrdered(ChannelGroup),
	///
	/// Выполняем команды строго по-порядку по группе
	///
	ReliableSequence(ChannelGroup),
}

///
/// Канал для отправки, отличается от [`ApplicationCommandChannelType`] полным набором данных для канала
///
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Channel {
	///
	/// Выполняем команды без учета порядка
	///
	ReliableUnordered,
	///
	/// Выполняем команды без учета порядка
	///
	UnreliableUnordered,
	///
	/// Отбрасываем команды из прошлого по группе
	///
	ReliableOrdered(ChannelGroup),
	///
	/// Отбрасываем команды из прошлого по группе
	///
	UnreliableOrdered(ChannelGroup),
	///
	/// Выполняем команды строго по-порядку по группе
	///
	ReliableSequence(ChannelGroup, ChannelSequence),
}

impl From<&Channel> for ChannelType {
	fn from(channel: &Channel) -> Self {
		match channel {
			Channel::ReliableUnordered => ChannelType::ReliableUnordered,
			Channel::ReliableOrdered(channel) => ChannelType::ReliableOrdered(*channel),
			Channel::UnreliableUnordered => ChannelType::UnreliableUnordered,
			Channel::UnreliableOrdered(channel) => ChannelType::UnreliableOrdered(*channel),
			Channel::ReliableSequence(channel, _) => ChannelType::ReliableSequence(*channel),
		}
	}
}

impl Channel {
	#[must_use]
	pub fn is_reliable(&self) -> bool {
		match self {
			Channel::ReliableUnordered => true,
			Channel::ReliableOrdered(_) => true,
			Channel::ReliableSequence(_, _) => true,
			Channel::UnreliableUnordered => false,
			Channel::UnreliableOrdered(_) => false,
		}
	}
	#[must_use]
	pub fn get_channel_group_id(&self) -> Option<ChannelGroup> {
		match self {
			Channel::ReliableUnordered => None,
			Channel::ReliableOrdered(group) => Some(*group),
			Channel::UnreliableUnordered => None,
			Channel::UnreliableOrdered(group) => Some(*group),
			Channel::ReliableSequence(group, _) => Some(*group),
		}
	}
}
