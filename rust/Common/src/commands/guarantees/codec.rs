use cheetah_game_realtime_protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use std::io::Cursor;
use std::num::TryFromIntError;

use thiserror::Error;

use crate::commands::context::CommandContextError;
use crate::commands::guarantees::{ChannelGroup, ChannelSequence, ReliabilityGuaranteesChannel};

///
/// Тип канала передача данных (тег для [`CommandChannel`])
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

impl ReliabilityGuaranteesChannel {
	///
	/// Получить идентификатор типа
	///
	#[must_use]
	pub fn get_type(&self) -> ChannelType {
		let id = match self {
			ReliabilityGuaranteesChannel::ReliableUnordered => ChannelType::RELIABLE_UNORDERED,
			ReliabilityGuaranteesChannel::ReliableOrdered(_, _) => ChannelType::RELIABLE_ORDERED,
			ReliabilityGuaranteesChannel::UnreliableUnordered => ChannelType::UNRELIABLE_UNORDERED,
			ReliabilityGuaranteesChannel::UnreliableOrdered(_, _) => ChannelType::UNRELIABLE_ORDERED,
			ReliabilityGuaranteesChannel::ReliableSequence(_, _) => ChannelType::RELIABLE_SEQUENCE,
		};
		assert!(id.0 < 8); // если больше 7 то надо переделывать формат передачи фреймов
		id
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			ReliabilityGuaranteesChannel::ReliableUnordered => {}
			ReliabilityGuaranteesChannel::UnreliableUnordered => {}
			ReliabilityGuaranteesChannel::ReliableOrdered(_, sequence) | ReliabilityGuaranteesChannel::UnreliableOrdered(_, sequence) | ReliabilityGuaranteesChannel::ReliableSequence(_, sequence) => {
				out.write_variable_u64(u64::from(sequence.0))?;
			}
		}
		Ok(())
	}

	pub fn decode(channel_type: &ChannelType, channel_group: Result<ChannelGroup, CommandContextError>, input: &mut Cursor<&[u8]>) -> Result<ReliabilityGuaranteesChannel, CommandChannelDecodeError> {
		Ok(match *channel_type {
			ChannelType::RELIABLE_UNORDERED => ReliabilityGuaranteesChannel::ReliableUnordered,
			ChannelType::UNRELIABLE_UNORDERED => ReliabilityGuaranteesChannel::UnreliableUnordered,
			ChannelType::RELIABLE_ORDERED => ReliabilityGuaranteesChannel::ReliableOrdered(channel_group?, ChannelSequence(input.read_variable_u64()?.try_into()?)),
			ChannelType::UNRELIABLE_ORDERED => ReliabilityGuaranteesChannel::UnreliableOrdered(channel_group?, ChannelSequence(input.read_variable_u64()?.try_into()?)),
			ChannelType::RELIABLE_SEQUENCE => ReliabilityGuaranteesChannel::ReliableSequence(channel_group?, ChannelSequence(input.read_variable_u64()?.try_into()?)),
			_ => return Err(CommandChannelDecodeError::UnknownType(*channel_type)),
		})
	}
}

#[derive(Error, Debug)]
pub enum CommandChannelDecodeError {
	#[error("Unknown command type {0:?}")]
	UnknownType(ChannelType),
	#[error("Io error {0}")]
	Io(#[from] std::io::Error),
	#[error("CommandContext error {0}")]
	CommandContext(#[from] CommandContextError),
	#[error("InputValueIsTooLarge {0}")]
	InputValueIsTooLarge(#[from] TryFromIntError),
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::commands::context::CommandContextError;
	use crate::commands::guarantees::codec::ChannelType;
	use crate::commands::guarantees::{ChannelGroup, ChannelSequence, ReliabilityGuaranteesChannel};

	#[test]
	fn test_reliable_unordered() {
		check(
			ReliabilityGuaranteesChannel::ReliableUnordered,
			ChannelType::RELIABLE_UNORDERED,
			Err(CommandContextError::ContextNotContainsChannelGroupId),
		);
	}

	#[test]
	fn test_reliable_ordered_by_group() {
		check(
			ReliabilityGuaranteesChannel::ReliableOrdered(ChannelGroup(100), ChannelSequence(255)),
			ChannelType::RELIABLE_ORDERED,
			Ok(ChannelGroup(100)),
		);
	}

	#[test]
	fn test_unreliable_unordered() {
		check(
			ReliabilityGuaranteesChannel::UnreliableUnordered,
			ChannelType::UNRELIABLE_UNORDERED,
			Err(CommandContextError::ContextNotContainsChannelGroupId),
		);
	}

	#[test]
	fn test_unreliable_ordered_by_group() {
		check(
			ReliabilityGuaranteesChannel::UnreliableOrdered(ChannelGroup(155), ChannelSequence(255)),
			ChannelType::UNRELIABLE_ORDERED,
			Ok(ChannelGroup(155)),
		);
	}

	#[test]
	fn test_reliable_sequence_by_group() {
		check(
			ReliabilityGuaranteesChannel::ReliableSequence(ChannelGroup(7), ChannelSequence(255)),
			ChannelType::RELIABLE_SEQUENCE,
			Ok(ChannelGroup(7)),
		);
	}

	fn check(original: ReliabilityGuaranteesChannel, channel_type: ChannelType, channel_group_id: Result<ChannelGroup, CommandContextError>) {
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		original.encode(&mut cursor).unwrap();
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = ReliabilityGuaranteesChannel::decode(&channel_type, channel_group_id, &mut read_cursor).unwrap();
		assert_eq!(write_position, read_cursor.position()); // проверяем что прочитаны все данные
		assert_eq!(original, actual);
	}
}
