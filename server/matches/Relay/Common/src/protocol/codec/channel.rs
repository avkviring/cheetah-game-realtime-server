use std::io::Cursor;

use thiserror::Error;

use crate::protocol::codec::commands::context::CommandContextError;
use crate::protocol::codec::cursor::VariableInt;
use crate::protocol::frame::applications::{ChannelGroup, ChannelSequence};
use crate::protocol::frame::channel::Channel;

///
/// Тип канала передача данных (тег для [CommandChannel])
///
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct ChannelType(pub u8);

impl ChannelType {
	const RELIABLE_UNORDERED: Self = ChannelType(0);
	const RELIABLE_ORDERED_BY_OBJECT: Self = ChannelType(1);
	const RELIABLE_ORDERED_BY_GROUP: Self = ChannelType(2);
	const UNRELIABLE_UNORDERED: Self = ChannelType(3);
	const UNRELIABLE_ORDERED_BY_OBJECT: Self = ChannelType(4);
	const UNRELIABLE_ORDERED_BY_GROUP: Self = ChannelType(5);
	const RELIABLE_SEQUENCE_BY_OBJECT: Self = ChannelType(6);
	const RELIABLE_SEQUENCE_BY_GROUP: Self = ChannelType(7);
}

///
/// Кодирование/декодирование канала
///

impl Channel {
	///
	/// Получить идентификатор типа
	///
	pub fn get_type(&self) -> ChannelType {
		let id = match self {
			Channel::ReliableUnordered => ChannelType::RELIABLE_UNORDERED,
			Channel::ReliableOrderedByObject => ChannelType::RELIABLE_ORDERED_BY_OBJECT,
			Channel::ReliableOrderedByGroup(_) => ChannelType::RELIABLE_ORDERED_BY_GROUP,
			Channel::UnreliableUnordered => ChannelType::UNRELIABLE_UNORDERED,
			Channel::UnreliableOrderedByObject => ChannelType::UNRELIABLE_ORDERED_BY_OBJECT,
			Channel::UnreliableOrderedByGroup(_) => ChannelType::UNRELIABLE_ORDERED_BY_GROUP,
			Channel::ReliableSequenceByObject(_) => ChannelType::RELIABLE_SEQUENCE_BY_OBJECT,
			Channel::ReliableSequenceByGroup(_, _) => ChannelType::RELIABLE_SEQUENCE_BY_GROUP,
		};
		assert!(id.0 < 8); // если больше 7 то надо переделывать формат передачи фреймов
		id
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			Channel::ReliableSequenceByObject(sequence) => out.write_variable_u64(*sequence as u64)?,
			Channel::ReliableSequenceByGroup(_, sequence) => out.write_variable_u64(*sequence as u64)?,
			_ => {}
		};
		Ok(())
	}

	pub fn decode(
		channel_type: &ChannelType,
		channel_group: Result<ChannelGroup, CommandContextError>,
		input: &mut Cursor<&mut [u8]>,
	) -> Result<Channel, CommandChannelDecodeError> {
		Ok(match *channel_type {
			ChannelType::RELIABLE_UNORDERED => Channel::ReliableUnordered,
			ChannelType::RELIABLE_ORDERED_BY_OBJECT => Channel::ReliableOrderedByObject,
			ChannelType::UNRELIABLE_UNORDERED => Channel::UnreliableUnordered,
			ChannelType::UNRELIABLE_ORDERED_BY_OBJECT => Channel::UnreliableOrderedByObject,
			ChannelType::RELIABLE_ORDERED_BY_GROUP => Channel::ReliableOrderedByGroup(channel_group?),
			ChannelType::UNRELIABLE_ORDERED_BY_GROUP => Channel::UnreliableOrderedByGroup(channel_group?),
			ChannelType::RELIABLE_SEQUENCE_BY_GROUP => {
				Channel::ReliableSequenceByGroup(channel_group?, input.read_variable_u64()? as ChannelSequence)
			}
			ChannelType::RELIABLE_SEQUENCE_BY_OBJECT => {
				Channel::ReliableSequenceByObject(input.read_variable_u64()? as ChannelSequence)
			}
			_ => return Err(CommandChannelDecodeError::UnknownType(*channel_type)),
		})
	}
}

#[derive(Error, Debug)]
pub enum CommandChannelDecodeError {
	#[error("Unknown command type {:?}", .0)]
	UnknownType(ChannelType),
	#[error("IO error {:?}", .source)]
	Io {
		#[from]
		source: std::io::Error,
	},
	#[error("CommandContext error {:?}", .source)]
	CommandContext {
		#[from]
		source: CommandContextError,
	},
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::protocol::codec::channel::ChannelType;
	use crate::protocol::codec::commands::context::CommandContextError;
	use crate::protocol::frame::applications::ChannelGroup;
	use crate::protocol::frame::channel::Channel;

	#[test]
	fn test_reliable_unordered() {
		check(
			Channel::ReliableUnordered,
			ChannelType::RELIABLE_UNORDERED,
			Result::Err(CommandContextError::ContextNotContainsChannelGroupId),
		);
	}

	#[test]
	fn test_reliable_ordered_by_object() {
		check(
			Channel::ReliableOrderedByObject,
			ChannelType::RELIABLE_ORDERED_BY_OBJECT,
			Result::Err(CommandContextError::ContextNotContainsChannelGroupId),
		);
	}

	#[test]
	fn test_reliable_ordered_by_group() {
		check(
			Channel::ReliableOrderedByGroup(100),
			ChannelType::RELIABLE_ORDERED_BY_GROUP,
			Result::Ok(100),
		);
	}

	#[test]
	fn test_unreliable_unordered() {
		check(
			Channel::UnreliableUnordered,
			ChannelType::UNRELIABLE_UNORDERED,
			Result::Err(CommandContextError::ContextNotContainsChannelGroupId),
		);
	}

	#[test]
	fn test_unreliable_ordered_by_object() {
		check(
			Channel::UnreliableOrderedByObject,
			ChannelType::UNRELIABLE_ORDERED_BY_OBJECT,
			Result::Err(CommandContextError::ContextNotContainsChannelGroupId),
		);
	}

	#[test]
	fn test_unreliable_ordered_by_group() {
		check(
			Channel::UnreliableOrderedByGroup(155),
			ChannelType::UNRELIABLE_ORDERED_BY_GROUP,
			Result::Ok(155),
		);
	}

	#[test]
	fn test_reliable_sequence_by_object() {
		check(
			Channel::ReliableSequenceByObject(255),
			ChannelType::RELIABLE_SEQUENCE_BY_OBJECT,
			Result::Err(CommandContextError::ContextNotContainsChannelGroupId),
		);
	}
	#[test]
	fn test_reliable_sequence_by_group() {
		check(
			Channel::ReliableSequenceByGroup(7, 255),
			ChannelType::RELIABLE_SEQUENCE_BY_GROUP,
			Result::Ok(7),
		);
	}

	fn check(original: Channel, channel_type: ChannelType, channel_group_id: Result<ChannelGroup, CommandContextError>) {
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		original.encode(&mut cursor).unwrap();
		let position = cursor.position();
		cursor.set_position(0);
		let actual = Channel::decode(&channel_type, channel_group_id, &mut cursor).unwrap();
		assert_eq!(cursor.position(), position); // проверяем что прочитаны все данные
		assert_eq!(original, actual);
	}
}
