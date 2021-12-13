use std::io::Cursor;

use thiserror::Error;

use crate::protocol::codec::commands::context::{CommandContext, CommandContextError};
use crate::protocol::codec::cursor::VariableInt;
use crate::protocol::frame::applications::ChannelSequence;
use crate::protocol::frame::channel::CommandChannel;

#[derive(Debug)]
pub struct ChannelTypeId(pub u8);

///
/// Кодирование/декодирование канала
///

impl CommandChannel {
	///
	/// Получить идентификатор типа
	///
	pub fn get_type_id(&self) -> ChannelTypeId {
		let id = match self {
			CommandChannel::ReliableUnordered => ChannelTypeId(0),
			CommandChannel::ReliableOrderedByObject => ChannelTypeId(1),
			CommandChannel::ReliableOrderedByGroup(_) => ChannelTypeId(2),
			CommandChannel::UnreliableUnordered => ChannelTypeId(3),
			CommandChannel::UnreliableOrderedByObject => ChannelTypeId(4),
			CommandChannel::UnreliableOrderedByGroup(_) => ChannelTypeId(5),
			CommandChannel::ReliableSequenceByObject(_) => ChannelTypeId(6),
			CommandChannel::ReliableSequenceByGroup(_, _) => ChannelTypeId(7),
		};
		assert!(id.0 < 8); // если больше 7 то надо переделывать формат передачи фреймов
		id
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			CommandChannel::ReliableUnordered => Ok(()),
			CommandChannel::ReliableOrderedByObject => Ok(()),
			CommandChannel::ReliableOrderedByGroup(group) => out.write_variable_u64(*group as u64),
			CommandChannel::UnreliableUnordered => Ok(()),
			CommandChannel::UnreliableOrderedByObject => Ok(()),
			CommandChannel::UnreliableOrderedByGroup(group) => out.write_variable_u64(*group as u64),
			CommandChannel::ReliableSequenceByObject(sequence) => out.write_variable_u64(*sequence as u64),
			CommandChannel::ReliableSequenceByGroup(group, sequence) => {
				out.write_variable_u64(*group as u64)?;
				out.write_variable_u64(*sequence as u64)
			}
		}
	}

	pub fn decode(
		channel_type_id: ChannelTypeId,
		context: &CommandContext,
		input: &mut Cursor<&mut [u8]>,
	) -> Result<CommandChannel, ApplicationCommandChannelDecodeError> {
		match channel_type_id {
			ChannelTypeId(0) => return Ok(CommandChannel::ReliableUnordered),
			ChannelTypeId(1) => return Ok(CommandChannel::ReliableOrderedByObject),
			ChannelTypeId(3) => return Ok(CommandChannel::UnreliableUnordered),
			ChannelTypeId(4) => return Ok(CommandChannel::UnreliableOrderedByObject),
			_ => {}
		};
		let channel_group_id = context.get_channel_group_id()?;
		match channel_type_id {
			ChannelTypeId(2) => return Ok(CommandChannel::ReliableOrderedByGroup(channel_group_id)),
			ChannelTypeId(5) => return Ok(CommandChannel::UnreliableOrderedByGroup(channel_group_id)),
			_ => {}
		}
		let channel_sequence = input.read_variable_u64()? as ChannelSequence;
		match channel_type_id {
			ChannelTypeId(7) => Ok(CommandChannel::ReliableSequenceByGroup(channel_group_id, channel_sequence)),
			ChannelTypeId(6) => Ok(CommandChannel::ReliableSequenceByObject(channel_sequence)),
			_ => Err(ApplicationCommandChannelDecodeError::UnknownType(channel_type_id)),
		}
	}
}

#[derive(Error, Debug)]
pub enum ApplicationCommandChannelDecodeError {
	#[error("Unknown command type {:?}", .0)]
	UnknownType(ChannelTypeId),
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
