use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::commands::field::FieldId;
use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::room::object::GameObjectId;

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

///
/// Установка значения new если текущее равно current
/// reset - значение после выхода пользователя
///
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct CompareAndSetLongCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub current: i64,
	pub new: i64,
	has_reset: bool,
	reset: i64,
}

impl IncrementLongC2SCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_i64(self.increment)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let increment = input.read_variable_i64()?;
		Ok(Self {
			object_id,
			field_id,
			increment,
		})
	}
}

impl CompareAndSetLongCommand {
	pub fn new(object_id: GameObjectId, field_id: FieldId, current: i64, new: i64, reset: Option<i64>) -> Self {
		Self {
			object_id,
			field_id,
			current,
			new,
			has_reset: reset.is_some(),
			reset: reset.unwrap_or_default(),
		}
	}
	pub fn get_reset(&self) -> Option<i64> {
		self.has_reset.then(|| self.reset)
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_i64(self.current)?;
		out.write_variable_i64(self.new)?;
		out.write_u8(u8::from(self.get_reset().is_some()))?;
		if let Some(reset_value) = self.get_reset() {
			out.write_variable_i64(reset_value)
		} else {
			Ok(())
		}
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let current = input.read_variable_i64()?;
		let new = input.read_variable_i64()?;
		let has_reset = input.read_u8()? == 1;
		#[allow(clippy::if_then_some_else_none)]
		let reset = has_reset.then(|| input.read_variable_i64()).unwrap_or_else(|| Ok(0))?;
		Ok(Self {
			object_id,
			field_id,
			current,
			new,
			has_reset,
			reset,
		})
	}
}
