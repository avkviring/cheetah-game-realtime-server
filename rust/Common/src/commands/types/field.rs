use crate::room::field::{FieldId, FieldType};
use std::fmt::Debug;
use std::io::Cursor;

use crate::room::object::GameObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct DeleteField {
	pub field_id: FieldId,
	pub object_id: GameObjectId,
	pub field_type: FieldType,
}

impl DeleteField {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		self.field_type.encode(out)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		Ok(DeleteField {
			field_id,
			object_id,
			field_type: FieldType::decode(input)?,
		})
	}
}
