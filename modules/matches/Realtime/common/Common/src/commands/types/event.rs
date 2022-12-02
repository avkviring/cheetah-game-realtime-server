use std::io::{Cursor, Error, ErrorKind};

use crate::commands::binary_value::BinaryValue;
use crate::commands::field::FieldId;
use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::room::object::GameObjectId;
use crate::room::RoomMemberId;

///
/// Событие по объекту
/// - C->S, S->C
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub event: BinaryValue,
}

///
/// Событие по объекту для определенного пользователя
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetEventCommand {
	pub target: RoomMemberId,
	pub event: EventCommand,
}

impl EventCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		self.event.encode(out)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		Ok(Self {
			object_id,
			field_id,
			event: BinaryValue::decode(input)?,
		})
	}
}

impl TargetEventCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(u64::from(self.target))?;
		self.event.encode(out)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let target = input.read_variable_u64()?.try_into().map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

		Ok(Self {
			target,
			event: EventCommand::decode(object_id, field_id, input)?,
		})
	}
}
