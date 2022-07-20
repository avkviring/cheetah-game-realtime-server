use std::io::Cursor;

use crate::commands::{field_type::ToFieldType, FieldType, FieldValue};
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

	pub fn decode(
		object_id: GameObjectId,
		field_id: FieldId,
		input: &mut Cursor<&[u8]>,
	) -> std::io::Result<Self> {
		Ok(DeleteFieldCommand {
			field_id,
			object_id,
			field_type: FieldType::decode(input)?,
		})
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct SetFieldCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub value: FieldValue,
}

impl SetFieldCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		self.value.encode(out)
	}

	pub fn decode<T: Into<FieldValue> + ToFieldType>(
		object_id: GameObjectId,
		field_id: FieldId,
		input: &mut Cursor<&[u8]>,
	) -> std::io::Result<Self> {
		let value = FieldValue::decode::<T>(input)?;
		Ok(Self {
			object_id,
			field_id,
			value,
		})
	}
}
