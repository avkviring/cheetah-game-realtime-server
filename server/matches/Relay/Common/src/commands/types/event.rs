use std::io::{Cursor, Error, ErrorKind, Read, Write};

use crate::commands::CommandBuffer;
use crate::constants::FieldId;
use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::room::object::GameObjectId;
use crate::room::RoomMemberId;

///
/// Событие по объекту
/// - C->S, S->C
#[derive(Debug, Clone, PartialEq)]
pub struct EventCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub event: CommandBuffer,
}

///
/// Событие по объекту для определенного пользователя
///
#[derive(Debug, Clone, PartialEq)]
pub struct TargetEventCommand {
	pub target: RoomMemberId,
	pub event: EventCommand,
}

impl EventCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.event.len() as u64)?;
		out.write_all(self.event.as_slice())
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let size = input.read_variable_u64()? as usize;
		let mut event = CommandBuffer::new();
		if size > event.capacity() {
			return Err(Error::new(
				ErrorKind::InvalidData,
				format!("Event buffer size to big {}", size),
			));
		}
		unsafe {
			event.set_len(size);
		}
		input.read_exact(&mut event[0..size])?;

		Ok(Self {
			object_id,
			field_id,
			event,
		})
	}
}

impl TargetEventCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.target as u64)?;
		self.event.encode(out)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let target = input.read_variable_u64()? as RoomMemberId;

		Ok(Self {
			target,
			event: EventCommand::decode(object_id, field_id, input)?,
		})
	}
}
