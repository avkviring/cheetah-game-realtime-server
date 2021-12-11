use std::io::Cursor;

use byteorder::{BigEndian, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::protocol::codec::cursor::VarInt;
use crate::protocol::frame::applications::{ChannelGroupId, ChannelSequence};

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
			ApplicationCommandChannel::ReliableOrderedByGroup(channel) => {
				ApplicationCommandChannelType::ReliableOrderedByGroup(*channel)
			}
			ApplicationCommandChannel::UnreliableUnordered => ApplicationCommandChannelType::UnreliableUnordered,
			ApplicationCommandChannel::UnreliableOrderedByObject => ApplicationCommandChannelType::UnreliableOrderedByObject,
			ApplicationCommandChannel::UnreliableOrderedByGroup(channel) => {
				ApplicationCommandChannelType::UnreliableOrderedByGroup(*channel)
			}
			ApplicationCommandChannel::ReliableSequenceByObject(_) => ApplicationCommandChannelType::ReliableSequenceByObject,
			ApplicationCommandChannel::ReliableSequenceByGroup(channel, _) => {
				ApplicationCommandChannelType::ReliableSequenceByGroup(*channel)
			}
		}
	}
}

impl ApplicationCommandChannel {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			ApplicationCommandChannel::ReliableUnordered => out.write_u8(1),
			ApplicationCommandChannel::ReliableOrderedByObject => out.write_u8(2),
			ApplicationCommandChannel::ReliableOrderedByGroup(group) => {
				out.write_u8(3)?;
				out.write_varint(*group as u64)
			}
			ApplicationCommandChannel::UnreliableUnordered => out.write_u8(4),
			ApplicationCommandChannel::UnreliableOrderedByObject => out.write_u8(5),
			ApplicationCommandChannel::UnreliableOrderedByGroup(group) => {
				out.write_u8(6)?;
				out.write_varint(*group as u64)
			}
			ApplicationCommandChannel::ReliableSequenceByObject(sequence) => {
				out.write_u8(7)?;
				out.write_varint(*sequence as u64)
			}
			ApplicationCommandChannel::ReliableSequenceByGroup(group, sequence) => {
				out.write_u8(8)?;
				out.write_varint(*group as u64)?;
				out.write_varint(*sequence as u64)
			}
		}
	}
}
