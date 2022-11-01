use std::io::Cursor;

use thiserror::Error;

use crate::protocol::codec::commands::context::CommandContextError;
use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::protocol::frame::applications::{ChannelGroup, ChannelSequence};
use crate::protocol::frame::channel::Channel;

///
/// Тип канала передача данных (тег для [CommandChannel])
///
#[derive(Debug, Eq, PartialEq, PartialOrd, Copy, Clone)]
pub struct ChannelType(pub u8);

impl ChannelType {
	const RELIABLE_UNORDERED: Self = ChannelType(0);
	const RELIABLE_ORDERED: Self = ChannelType(1);
	const UNRELIABLE_UNORDERED: Self = ChannelType(2);
	const UNRELIABLE_ORDERED: Self = ChannelType(3);
	const RELIABLE_SEQUENCE: Self = ChannelType(4);
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
			Channel::ReliableOrdered(_) => ChannelType::RELIABLE_ORDERED,
			Channel::UnreliableUnordered => ChannelType::UNRELIABLE_UNORDERED,
			Channel::UnreliableOrdered(_) => ChannelType::UNRELIABLE_ORDERED,
			Channel::ReliableSequence(_, _) => ChannelType::RELIABLE_SEQUENCE,
		};
		assert!(id.0 < 8); // если больше 7 то надо переделывать формат передачи фреймов
		id
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		if let Channel::ReliableSequence(_, sequence) = self {
			out.write_variable_u64(sequence.0 as u64)?
		};
		Ok(())
	}

	pub fn decode(
		channel_type: &ChannelType,
		channel_group: Result<ChannelGroup, CommandContextError>,
		input: &mut Cursor<&[u8]>,
	) -> Result<Channel, CommandChannelDecodeError> {
		Ok(match *channel_type {
			ChannelType::RELIABLE_UNORDERED => Channel::ReliableUnordered,
			ChannelType::UNRELIABLE_UNORDERED => Channel::UnreliableUnordered,
			ChannelType::RELIABLE_ORDERED => Channel::ReliableOrdered(channel_group?),
			ChannelType::UNRELIABLE_ORDERED => Channel::UnreliableOrdered(channel_group?),
			ChannelType::RELIABLE_SEQUENCE => Channel::ReliableSequence(channel_group?, ChannelSequence(input.read_variable_u64()? as u32)),
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
	use crate::protocol::frame::applications::{ChannelGroup, ChannelSequence};
	use crate::protocol::frame::channel::Channel;

	#[test]
	fn test_reliable_unordered() {
		check(
			Channel::ReliableUnordered,
			ChannelType::RELIABLE_UNORDERED,
			Err(CommandContextError::ContextNotContainsChannelGroupId),
		);
	}

	#[test]
	fn test_reliable_ordered_by_group() {
		check(
			Channel::ReliableOrdered(ChannelGroup(100)),
			ChannelType::RELIABLE_ORDERED,
			Ok(ChannelGroup(100)),
		);
	}

	#[test]
	fn test_unreliable_unordered() {
		check(
			Channel::UnreliableUnordered,
			ChannelType::UNRELIABLE_UNORDERED,
			Err(CommandContextError::ContextNotContainsChannelGroupId),
		);
	}

	#[test]
	fn test_unreliable_ordered_by_group() {
		check(
			Channel::UnreliableOrdered(ChannelGroup(155)),
			ChannelType::UNRELIABLE_ORDERED,
			Ok(ChannelGroup(155)),
		);
	}

	#[test]
	fn test_reliable_sequence_by_group() {
		check(
			Channel::ReliableSequence(ChannelGroup(7), ChannelSequence(255)),
			ChannelType::RELIABLE_SEQUENCE,
			Ok(ChannelGroup(7)),
		);
	}

	fn check(original: Channel, channel_type: ChannelType, channel_group_id: Result<ChannelGroup, CommandContextError>) {
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		original.encode(&mut cursor).unwrap();
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = Channel::decode(&channel_type, channel_group_id, &mut read_cursor).unwrap();
		assert_eq!(write_position, read_cursor.position()); // проверяем что прочитаны все данные
		assert_eq!(original, actual);
	}
}
