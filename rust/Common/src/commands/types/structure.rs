use std::io::Cursor;

use crate::commands::binary_value::Buffer;
use crate::commands::field::FieldId;
use crate::room::object::GameObjectId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct SetStructureCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub value: Buffer,
}

impl SetStructureCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		self.value.encode(out)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let value = Buffer::decode(input)?;
		Ok(SetStructureCommand { object_id, field_id, value })
	}
}
