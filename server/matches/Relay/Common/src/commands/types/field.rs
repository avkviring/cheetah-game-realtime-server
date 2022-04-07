use std::io::Cursor;

use crate::commands::FieldType;
use crate::constants::FieldId;
use crate::room::object::GameObjectId;

#[derive(Debug, PartialEq, Clone)]
pub struct DeleteFieldCommand {
	pub field_id: FieldId,
	pub object_id: GameObjectId,
	pub field_type: FieldType,
}

impl DeleteFieldCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		self.field_type.encode(out)
	}
	pub fn decode(field_id: FieldId, object_id: GameObjectId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		Ok(DeleteFieldCommand {
			field_id,
			object_id,
			field_type: FieldType::decode(input)?,
		})
	}
}
