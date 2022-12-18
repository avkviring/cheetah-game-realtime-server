use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::commands::binary_value::BinaryValue;
use crate::commands::field::FieldId;
use crate::room::object::GameObjectId;

///
/// Обновить структуру в обьекте
/// - C->S, S->C
///

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct CompareAndSetStructureCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub current: BinaryValue,
	pub new: BinaryValue,
	has_reset: bool,
	reset: BinaryValue,
}

impl CompareAndSetStructureCommand {
	pub fn new(object_id: GameObjectId, field_id: FieldId, current: BinaryValue, new: BinaryValue, reset: Option<BinaryValue>) -> Self {
		Self {
			object_id,
			field_id,
			current,
			new,
			has_reset: reset.is_some(),
			reset: reset.unwrap_or_default(),
		}
	}
	pub fn get_reset(&self) -> Option<&BinaryValue> {
		self.has_reset.then(|| &self.reset)
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		self.current.encode(out)?;
		self.new.encode(out)?;
		match &self.get_reset() {
			None => {
				out.write_u8(0)?;
			}
			Some(reset) => {
				out.write_u8(1)?;
				reset.encode(out)?;
			}
		};
		Ok(())
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let current = BinaryValue::decode(input)?;
		let new = BinaryValue::decode(input)?;
		let has_reset = input.read_u8()? == 1;
		let reset = has_reset
			.then(|| BinaryValue::decode(input))
			.unwrap_or_else(|| Ok(BinaryValue::default()))?;
		#[allow(clippy::if_then_some_else_none)]
		Ok(CompareAndSetStructureCommand {
			object_id,
			field_id,
			current,
			new,
			has_reset,
			reset,
		})
	}
}
