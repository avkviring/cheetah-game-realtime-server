use crate::protocol::frame::applications::{ChannelGroup, ChannelSequence};

///
/// Тип гарантий
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReliabilityGuarantees {
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

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ReliabilityGuaranteesChannel {
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

impl From<&ReliabilityGuaranteesChannel> for ReliabilityGuarantees {
	fn from(channel: &ReliabilityGuaranteesChannel) -> Self {
		match channel {
			ReliabilityGuaranteesChannel::ReliableUnordered => ReliabilityGuarantees::ReliableUnordered,
			ReliabilityGuaranteesChannel::ReliableOrdered(channel) => ReliabilityGuarantees::ReliableOrdered(*channel),
			ReliabilityGuaranteesChannel::UnreliableUnordered => ReliabilityGuarantees::UnreliableUnordered,
			ReliabilityGuaranteesChannel::UnreliableOrdered(channel) => ReliabilityGuarantees::UnreliableOrdered(*channel),
			ReliabilityGuaranteesChannel::ReliableSequence(channel, _) => ReliabilityGuarantees::ReliableSequence(*channel),
		}
	}
}

impl ReliabilityGuaranteesChannel {
	#[must_use]
	pub fn is_reliable(&self) -> bool {
		match self {
			ReliabilityGuaranteesChannel::ReliableUnordered => true,
			ReliabilityGuaranteesChannel::ReliableOrdered(_) => true,
			ReliabilityGuaranteesChannel::ReliableSequence(_, _) => true,
			ReliabilityGuaranteesChannel::UnreliableUnordered => false,
			ReliabilityGuaranteesChannel::UnreliableOrdered(_) => false,
		}
	}
	#[must_use]
	pub fn get_channel_group_id(&self) -> Option<ChannelGroup> {
		match self {
			ReliabilityGuaranteesChannel::ReliableUnordered => None,
			ReliabilityGuaranteesChannel::ReliableOrdered(group) => Some(*group),
			ReliabilityGuaranteesChannel::UnreliableUnordered => None,
			ReliabilityGuaranteesChannel::UnreliableOrdered(group) => Some(*group),
			ReliabilityGuaranteesChannel::ReliableSequence(group, _) => Some(*group),
		}
	}
}
