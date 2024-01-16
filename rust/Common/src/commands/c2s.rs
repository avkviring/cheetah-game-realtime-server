use std::io::Cursor;

use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

use crate::commands::context::CommandContextError;
use crate::commands::types::create::{C2SCreatedGameObject, CreateGameObject};
use crate::commands::types::event::TargetEvent;
use crate::commands::types::field::DeleteField;
use crate::commands::types::float::{DoubleField, IncrementDouble};
use crate::commands::types::long::{IncrementLong, LongField};
use crate::commands::types::structure::BinaryField;
use crate::commands::{CommandDecodeError, CommandTypeId};
use crate::room::field::{Field, FieldId, FieldType};
use crate::room::object::GameObjectId;

#[derive(Debug, PartialEq, Clone, AsRefStr, Serialize, Deserialize)]
pub enum C2SCommand {
	CreateGameObject(CreateGameObject),
	CreatedGameObject(Box<C2SCreatedGameObject>),
	IncrementLongValue(IncrementLong),
	SetLong(LongField),
	SetDouble(DoubleField),
	SetStructure(Box<BinaryField>),
	IncrementDouble(IncrementDouble),
	Event(Box<BinaryField>),
	TargetEvent(Box<TargetEvent>),
	Delete(GameObjectId),
	DeleteField(DeleteField),
	AttachToRoom,
	DetachFromRoom,
	AddItem(Box<BinaryField>),
}

impl C2SCommand {
	#[must_use]
	pub fn get_field_id(&self) -> Option<FieldId> {
		match self {
			C2SCommand::CreateGameObject(_) => None,
			C2SCommand::CreatedGameObject(_) => None,
			C2SCommand::IncrementLongValue(command) => Some(command.field_id),
			C2SCommand::IncrementDouble(command) => Some(command.field_id),
			C2SCommand::Event(command) => Some(command.field_id),
			C2SCommand::TargetEvent(command) => Some(command.event.field_id),
			C2SCommand::Delete(_) => None,
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
			C2SCommand::DeleteField(command) => Some(command.field_id),
			C2SCommand::SetLong(command) => command.field_id.into(),
			C2SCommand::SetDouble(command) => command.field_id.into(),
			C2SCommand::SetStructure(command) => command.field_id.into(),
			C2SCommand::AddItem(command) => command.field_id.into(),
		}
	}
	#[must_use]
	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			C2SCommand::CreateGameObject(command) => Some(command.object_id),
			C2SCommand::CreatedGameObject(command) => Some(command.object_id),
			C2SCommand::IncrementLongValue(command) => Some(command.object_id),
			C2SCommand::IncrementDouble(command) => Some(command.object_id),
			C2SCommand::Event(command) => Some(command.object_id),
			C2SCommand::TargetEvent(command) => Some(command.event.object_id),
			C2SCommand::Delete(object_id) => Some(object_id.clone()),
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
			C2SCommand::DeleteField(command) => Some(command.object_id),
			C2SCommand::SetLong(command) => command.object_id.into(),
			C2SCommand::SetDouble(command) => command.object_id.into(),
			C2SCommand::SetStructure(command) => command.object_id.into(),
			C2SCommand::AddItem(command) => command.object_id.into(),
		}
	}

	#[must_use]
	pub fn get_field_type(&self) -> Option<FieldType> {
		match self {
			C2SCommand::CreateGameObject(_) => None,
			C2SCommand::CreatedGameObject(_) => None,
			C2SCommand::IncrementLongValue(_) => Some(FieldType::Long),
			C2SCommand::IncrementDouble(_) => Some(FieldType::Double),
			C2SCommand::Event(_) => Some(FieldType::Event),
			C2SCommand::TargetEvent(_) => Some(FieldType::Event),
			C2SCommand::Delete(_) => None,
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
			C2SCommand::DeleteField(command) => Some(command.field_type),
			C2SCommand::SetLong(_) => FieldType::Long.into(),
			C2SCommand::SetDouble(_) => FieldType::Double.into(),
			C2SCommand::SetStructure(_) => FieldType::Structure.into(),
			C2SCommand::AddItem(_) => FieldType::Items.into(),
		}
	}

	#[must_use]
	pub fn get_type_id(&self) -> CommandTypeId {
		match self {
			C2SCommand::CreateGameObject(_) => CommandTypeId::CreateGameObject,
			C2SCommand::CreatedGameObject(_) => CommandTypeId::CreatedGameObject,
			C2SCommand::IncrementLongValue(_) => CommandTypeId::IncrementLong,
			C2SCommand::IncrementDouble(_) => CommandTypeId::IncrementDouble,
			C2SCommand::Event(_) => CommandTypeId::SendEvent,
			C2SCommand::TargetEvent(_) => CommandTypeId::TargetEvent,
			C2SCommand::Delete(_) => CommandTypeId::DeleteObject,
			C2SCommand::AttachToRoom => CommandTypeId::AttachToRoom,
			C2SCommand::DetachFromRoom => CommandTypeId::DetachFromRoom,
			C2SCommand::DeleteField(_) => CommandTypeId::DeleteField,
			C2SCommand::SetLong(_) => CommandTypeId::SetLong,
			C2SCommand::SetDouble(_) => CommandTypeId::SetDouble,
			C2SCommand::SetStructure(_) => CommandTypeId::SetStructure,
			C2SCommand::AddItem(_) => CommandTypeId::AddItem,
		}
	}

	#[must_use]
	pub fn get_field(&self) -> Option<Field> {
		if let (Some(id), Some(field_type)) = (self.get_field_id(), self.get_field_type()) {
			Some(Field { id, field_type })
		} else {
			None
		}
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			C2SCommand::CreateGameObject(command) => command.encode(out),
			C2SCommand::CreatedGameObject(command) => command.encode(out),
			C2SCommand::IncrementLongValue(command) => command.encode(out),
			C2SCommand::IncrementDouble(command) => command.encode(out),
			C2SCommand::Event(command) => command.encode(out),
			C2SCommand::TargetEvent(command) => command.encode(out),
			C2SCommand::Delete(_) => Ok(()),
			C2SCommand::AttachToRoom => Ok(()),
			C2SCommand::DetachFromRoom => Ok(()),
			C2SCommand::DeleteField(command) => command.encode(out),
			C2SCommand::SetLong(command) => command.encode(out),
			C2SCommand::SetDouble(command) => command.encode(out),
			C2SCommand::SetStructure(command) => command.encode(out),
			C2SCommand::AddItem(command) => command.encode(out),
		}
	}

	pub(crate) fn decode(
		command_type_id: CommandTypeId,
		object_id: Result<GameObjectId, CommandContextError>,
		field_id: Result<FieldId, CommandContextError>,
		input: &mut Cursor<&[u8]>,
	) -> Result<C2SCommand, CommandDecodeError> {
		Ok(match command_type_id {
			CommandTypeId::AttachToRoom => C2SCommand::AttachToRoom,
			CommandTypeId::DetachFromRoom => C2SCommand::DetachFromRoom,
			CommandTypeId::CreatedGameObject => C2SCommand::CreatedGameObject(C2SCreatedGameObject::decode(object_id?, input)?.into()),
			CommandTypeId::DeleteObject => C2SCommand::Delete(object_id?.clone()),
			CommandTypeId::CreateGameObject => C2SCommand::CreateGameObject(CreateGameObject::decode(object_id?, input)?),
			CommandTypeId::IncrementLong => C2SCommand::IncrementLongValue(IncrementLong::decode(object_id?, field_id?, input)?),
			CommandTypeId::IncrementDouble => C2SCommand::IncrementDouble(IncrementDouble::decode(object_id?, field_id?, input)?),
			CommandTypeId::SetDouble => C2SCommand::SetDouble(DoubleField::decode(object_id?, field_id?, input)?),
			CommandTypeId::SetLong => C2SCommand::SetLong(LongField::decode(object_id?, field_id?, input)?),
			CommandTypeId::SetStructure => C2SCommand::SetStructure(BinaryField::decode(object_id?, field_id?, input)?.into()),
			CommandTypeId::SendEvent => C2SCommand::Event(BinaryField::decode(object_id?, field_id?, input)?.into()),
			CommandTypeId::TargetEvent => C2SCommand::TargetEvent(TargetEvent::decode(object_id?, field_id?, input)?.into()),
			CommandTypeId::DeleteField => C2SCommand::DeleteField(DeleteField::decode(object_id?, field_id?, input)?),
			CommandTypeId::MemberConnected => return Err(CommandDecodeError::UnknownTypeId(command_type_id)),
			CommandTypeId::MemberDisconnected => return Err(CommandDecodeError::UnknownTypeId(command_type_id)),
			CommandTypeId::AddItem => C2SCommand::AddItem(BinaryField::decode(object_id?, field_id?, input)?.into()),
		})
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::commands::c2s::C2SCommand;
	use crate::commands::context::CommandContextError;
	use crate::commands::types::create::{C2SCreatedGameObject, CreateGameObject};
	use crate::commands::types::event::TargetEvent;
	use crate::commands::types::float::{DoubleField, IncrementDouble};
	use crate::commands::types::long::{IncrementLong, LongField};
	use crate::commands::types::structure::BinaryField;
	use crate::commands::CommandTypeId;
	use crate::room::access::AccessGroups;
	use crate::room::buffer::Buffer;
	use crate::room::field::FieldId;
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	#[test]
	fn should_decode_encode_attach() {
		check(&C2SCommand::AttachToRoom, CommandTypeId::AttachToRoom, None, None);
	}

	#[test]
	fn should_decode_encode_detach() {
		check(&C2SCommand::DetachFromRoom, CommandTypeId::DetachFromRoom, None, None);
	}

	#[test]
	fn should_decode_encode_create_member_object() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			&C2SCommand::CreateGameObject(CreateGameObject {
				object_id,
				template: 3,
				access_groups: AccessGroups(5),
			}),
			CommandTypeId::CreateGameObject,
			Some(object_id),
			None,
		);
	}

	#[test]
	fn should_decode_encode_created() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			&C2SCommand::CreatedGameObject(C2SCreatedGameObject::new(object_id, false, None).into()),
			CommandTypeId::CreatedGameObject,
			Some(object_id),
			None,
		);
	}

	#[test]
	fn should_decode_encode_set_long() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::SetLong(LongField { object_id, field_id, value: 100 }),
			CommandTypeId::SetLong,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_increment_long_value() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::IncrementLongValue(IncrementLong { object_id, field_id, increment: 100 }),
			CommandTypeId::IncrementLong,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_double() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::SetDouble(DoubleField { object_id, field_id, value: 3.15 }),
			CommandTypeId::SetDouble,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_incremental_double() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::IncrementDouble(IncrementDouble { object_id, field_id, increment: 3.15 }),
			CommandTypeId::IncrementDouble,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_structure() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::SetStructure(
				BinaryField {
					object_id,
					field_id,
					value: Buffer::from([1, 2, 3, 4].as_ref()),
				}
				.into(),
			),
			CommandTypeId::SetStructure,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_event() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::Event(
				BinaryField {
					object_id,
					field_id,
					value: Buffer::from(vec![1, 2, 3, 4].as_slice()),
				}
				.into(),
			),
			CommandTypeId::SendEvent,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_target_event() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::TargetEvent(
				TargetEvent {
					target: 10,
					event: BinaryField {
						object_id,
						field_id,
						value: Buffer::from(vec![1, 2, 3, 4].as_slice()),
					},
				}
				.into(),
			),
			CommandTypeId::TargetEvent,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_delete() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(&C2SCommand::Delete(object_id), CommandTypeId::DeleteObject, Some(object_id), None);
	}

	fn check(expected: &C2SCommand, command_type_id: CommandTypeId, object_id: Option<GameObjectId>, field_id: Option<FieldId>) {
		let object_id = object_id.ok_or(CommandContextError::ContextNotContainsObjectId);
		let field_id = field_id.ok_or(CommandContextError::ContextNotContainsFieldId);
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		expected.encode(&mut cursor).unwrap();
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = C2SCommand::decode(command_type_id, object_id, field_id, &mut read_cursor).unwrap();
		assert_eq!(write_position, read_cursor.position());
		assert_eq!(expected, &actual);
	}
}
