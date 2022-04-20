use std::io::Cursor;

use crate::commands::binary_value::BinaryValue;
use crate::constants::FieldId;
use crate::room::object::GameObjectId;

///
/// Обновить структуру в обьекте
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq)]
pub struct SetStructureCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub structure: BinaryValue,
}
impl SetStructureCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		self.structure.encode(out)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let structure = BinaryValue::decode(input)?;
		Ok(Self {
			object_id,
			field_id,
			structure,
		})
	}
}
