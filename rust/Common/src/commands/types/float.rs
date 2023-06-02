use std::io::Cursor;

use crate::room::field::FieldId;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::room::object::GameObjectId;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct SetDoubleCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub value: f64,
}

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq, Clone)]
#[repr(C)]
pub struct IncrementDoubleC2SCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub increment: f64,
}

impl SetDoubleCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_f64::<BigEndian>(self.value)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let value = input.read_f64::<BigEndian>()?;
		Ok(Self { object_id, field_id, value })
	}
}

impl IncrementDoubleC2SCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_f64::<BigEndian>(self.increment)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let increment = input.read_f64::<BigEndian>()?;
		Ok(Self { object_id, field_id, increment })
	}
}
