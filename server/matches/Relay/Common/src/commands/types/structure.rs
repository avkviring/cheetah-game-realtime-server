use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::commands::binary_value::BinaryValue;
use crate::constants::FieldId;
use crate::room::object::GameObjectId;

///
/// Обновить структуру в обьекте
/// - C->S, S->C
///

#[derive(Debug, Clone, PartialEq)]
pub struct CompareAndSetStructureCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub current: BinaryValue,
	pub new: BinaryValue,
	pub reset: Option<BinaryValue>,
}

impl CompareAndSetStructureCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		self.current.encode(out)?;
		self.new.encode(out)?;
		match &self.reset {
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
		Ok(CompareAndSetStructureCommand {
			object_id,
			field_id,
			current: BinaryValue::decode(input)?,
			new: BinaryValue::decode(input)?,
			reset: if input.read_u8()? == 1 {
				Some(BinaryValue::decode(input)?)
			} else {
				None
			},
		})
	}
}
