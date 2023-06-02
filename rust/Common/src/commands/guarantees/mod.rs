pub mod codec;

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, Default)]
#[repr(C)]
pub struct ChannelGroup(pub u8);

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, Default)]
#[repr(C)]
pub struct ChannelSequence(pub u32);

///
/// Тип гарантий
///
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
	/// Выполняем команды без учета порядка, гарантированная доставка
	///
	ReliableUnordered,
	///
	/// Выполняем команды без учета порядка
	///
	UnreliableUnordered,
	///
	/// Отбрасываем команды из прошлого по группе, гарантированная доставка
	///
	ReliableOrdered(ChannelGroup, ChannelSequence),
	///
	/// Отбрасываем команды из прошлого по группе
	///
	UnreliableOrdered(ChannelGroup, ChannelSequence),
	///
	/// Выполняем команды строго по-порядку по группе, гарантированная доставка
	///
	ReliableSequence(ChannelGroup, ChannelSequence),
}

impl From<&ReliabilityGuaranteesChannel> for ReliabilityGuarantees {
	fn from(channel: &ReliabilityGuaranteesChannel) -> Self {
		match channel {
			ReliabilityGuaranteesChannel::ReliableUnordered => ReliabilityGuarantees::ReliableUnordered,
			ReliabilityGuaranteesChannel::ReliableOrdered(channel, _) => ReliabilityGuarantees::ReliableOrdered(*channel),
			ReliabilityGuaranteesChannel::UnreliableUnordered => ReliabilityGuarantees::UnreliableUnordered,
			ReliabilityGuaranteesChannel::UnreliableOrdered(channel, _) => ReliabilityGuarantees::UnreliableOrdered(*channel),
			ReliabilityGuaranteesChannel::ReliableSequence(channel, _) => ReliabilityGuarantees::ReliableSequence(*channel),
		}
	}
}

impl ReliabilityGuaranteesChannel {
	#[must_use]
	pub fn is_reliable(&self) -> bool {
		match self {
			ReliabilityGuaranteesChannel::ReliableUnordered => true,
			ReliabilityGuaranteesChannel::ReliableOrdered(_, _) => true,
			ReliabilityGuaranteesChannel::ReliableSequence(_, _) => true,
			ReliabilityGuaranteesChannel::UnreliableUnordered => false,
			ReliabilityGuaranteesChannel::UnreliableOrdered(_, _) => false,
		}
	}
	#[must_use]
	pub fn get_channel_group_id(&self) -> Option<ChannelGroup> {
		match self {
			ReliabilityGuaranteesChannel::ReliableUnordered => None,
			ReliabilityGuaranteesChannel::ReliableOrdered(group, _) => Some(*group),
			ReliabilityGuaranteesChannel::UnreliableUnordered => None,
			ReliabilityGuaranteesChannel::UnreliableOrdered(group, _) => Some(*group),
			ReliabilityGuaranteesChannel::ReliableSequence(group, _) => Some(*group),
		}
	}
}
