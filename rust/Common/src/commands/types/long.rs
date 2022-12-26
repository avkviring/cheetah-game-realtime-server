use std::io::Cursor;

use crate::commands::field::FieldId;
use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::room::object::GameObjectId;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[repr(C)]
pub struct SetLongCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub value: i64,
}

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct IncrementLongC2SCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub increment: i64,
}

impl SetLongCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_i64(self.value)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let value = input.read_variable_i64()?;
		Ok(Self { object_id, field_id, value })
	}
}

impl IncrementLongC2SCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_i64(self.increment)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let increment = input.read_variable_i64()?;
		Ok(Self { object_id, field_id, increment })
	}
}
