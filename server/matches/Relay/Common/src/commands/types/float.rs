use std::io::Cursor;

use byteorder::{BigEndian, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::constants::FieldId;
use crate::room::object::GameObjectId;

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct IncrementFloat64C2SCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub increment: f64,
}

///
/// Установка счетчика
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetFloat64Command {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub value: f64,
}

impl SetFloat64Command {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_f64::<BigEndian>(self.value)
	}
}

impl IncrementFloat64C2SCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_f64::<BigEndian>(self.increment)
	}
}
