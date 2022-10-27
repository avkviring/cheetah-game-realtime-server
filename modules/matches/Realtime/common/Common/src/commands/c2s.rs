use std::io::Cursor;

use strum_macros::AsRefStr;

use crate::commands::field::FieldId;
use crate::commands::types::create::{C2SCreatedGameObjectCommand, CreateGameObjectCommand};
use crate::commands::types::delete::DeleteGameObjectCommand;
use crate::commands::types::event::{EventCommand, TargetEventCommand};
use crate::commands::types::field::{DeleteFieldCommand, SetFieldCommand};
use crate::commands::types::float::IncrementDoubleC2SCommand;
use crate::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand};
use crate::commands::types::structure::CompareAndSetStructureCommand;
use crate::commands::{CommandDecodeError, CommandTypeId, FieldType, FieldValue};
use crate::protocol::codec::commands::context::CommandContextError;
use crate::room::object::GameObjectId;

#[derive(Debug, PartialEq, Clone, AsRefStr)]
#[allow(clippy::large_enum_variant)]
pub enum C2SCommand {
	CreateGameObject(CreateGameObjectCommand),
	CreatedGameObject(C2SCreatedGameObjectCommand),
	IncrementLongValue(IncrementLongC2SCommand),
	CompareAndSetLong(CompareAndSetLongCommand),
	SetField(SetFieldCommand),
	IncrementDouble(IncrementDoubleC2SCommand),
	CompareAndSetStructure(CompareAndSetStructureCommand),
	Event(EventCommand),
	TargetEvent(TargetEventCommand),
	Delete(DeleteGameObjectCommand),
	DeleteField(DeleteFieldCommand),
	///
	/// Загрузить все объекты комнаты
	///
	AttachToRoom,
	DetachFromRoom,
}

impl C2SCommand {
	pub fn get_field_id(&self) -> Option<FieldId> {
		match self {
			C2SCommand::CreateGameObject(_) => None,
			C2SCommand::CreatedGameObject(_) => None,
			C2SCommand::SetField(command) => Some(command.field_id),
			C2SCommand::IncrementLongValue(command) => Some(command.field_id),
			C2SCommand::CompareAndSetLong(command) => Some(command.field_id),
			C2SCommand::CompareAndSetStructure(command) => Some(command.field_id),
			C2SCommand::IncrementDouble(command) => Some(command.field_id),
			C2SCommand::Event(command) => Some(command.field_id),
			C2SCommand::TargetEvent(command) => Some(command.event.field_id),
			C2SCommand::Delete(_) => None,
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
			C2SCommand::DeleteField(command) => Some(command.field_id),
		}
	}
	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			C2SCommand::CreateGameObject(command) => Some(command.object_id.clone()),
			C2SCommand::CreatedGameObject(command) => Some(command.object_id.clone()),
			C2SCommand::SetField(command) => Some(command.object_id.clone()),
			C2SCommand::IncrementLongValue(command) => Some(command.object_id.clone()),
			C2SCommand::CompareAndSetLong(command) => Some(command.object_id.clone()),
			C2SCommand::IncrementDouble(command) => Some(command.object_id.clone()),
			C2SCommand::Event(command) => Some(command.object_id.clone()),
			C2SCommand::TargetEvent(command) => Some(command.event.object_id.clone()),
			C2SCommand::Delete(command) => Some(command.object_id.clone()),
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
			C2SCommand::DeleteField(command) => Some(command.object_id.clone()),
			C2SCommand::CompareAndSetStructure(command) => Some(command.object_id.clone()),
		}
	}

	pub fn get_field_type(&self) -> Option<FieldType> {
		match self {
			C2SCommand::CreateGameObject(_) => None,
			C2SCommand::CreatedGameObject(_) => None,
			C2SCommand::SetField(command) => Some(command.value.field_type()),
			C2SCommand::IncrementLongValue(_) => Some(FieldType::Long),
			C2SCommand::CompareAndSetLong(_) => Some(FieldType::Long),
			C2SCommand::IncrementDouble(_) => Some(FieldType::Double),
			C2SCommand::Event(_) => Some(FieldType::Event),
			C2SCommand::TargetEvent(_) => Some(FieldType::Event),
			C2SCommand::Delete(_) => None,
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
			C2SCommand::DeleteField(command) => Some(command.field_type),
			C2SCommand::CompareAndSetStructure(_) => Some(FieldType::Structure),
		}
	}

	pub fn get_type_id(&self) -> CommandTypeId {
		match self {
			C2SCommand::CreateGameObject(_) => CommandTypeId::CREATE_GAME_OBJECT,
			C2SCommand::CreatedGameObject(_) => CommandTypeId::CREATED_GAME_OBJECT,
			C2SCommand::SetField(command) => match command.value {
				FieldValue::Long(_) => CommandTypeId::SET_LONG,
				FieldValue::Double(_) => CommandTypeId::SET_DOUBLE,
				FieldValue::Structure(_) => CommandTypeId::SET_STRUCTURE,
			},
			C2SCommand::IncrementLongValue(_) => CommandTypeId::INCREMENT_LONG,
			C2SCommand::CompareAndSetLong(_) => CommandTypeId::COMPARE_AND_SET_LONG,
			C2SCommand::CompareAndSetStructure(_) => CommandTypeId::COMPARE_AND_SET_STRUCTURE,
			C2SCommand::IncrementDouble(_) => CommandTypeId::INCREMENT_DOUBLE,
			C2SCommand::Event(_) => CommandTypeId::EVENT,
			C2SCommand::TargetEvent(_) => CommandTypeId::TARGET_EVENT,
			C2SCommand::Delete(_) => CommandTypeId::DELETE,
			C2SCommand::AttachToRoom => CommandTypeId::ATTACH_TO_ROOM,
			C2SCommand::DetachFromRoom => CommandTypeId::DETACH_FROM_ROOM,
			C2SCommand::DeleteField(_) => CommandTypeId::DELETE_FIELD,
		}
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			C2SCommand::CreateGameObject(command) => command.encode(out),
			C2SCommand::CreatedGameObject(command) => command.encode(out),
			C2SCommand::SetField(command) => command.encode(out),
			C2SCommand::IncrementLongValue(command) => command.encode(out),
			C2SCommand::CompareAndSetLong(command) => command.encode(out),
			C2SCommand::IncrementDouble(command) => command.encode(out),
			C2SCommand::Event(command) => command.encode(out),
			C2SCommand::TargetEvent(command) => command.encode(out),
			C2SCommand::Delete(_) => Ok(()),
			C2SCommand::AttachToRoom => Ok(()),
			C2SCommand::DetachFromRoom => Ok(()),
			C2SCommand::DeleteField(command) => command.encode(out),
			C2SCommand::CompareAndSetStructure(command) => command.encode(out),
		}
	}

	pub(crate) fn decode(
		command_type_id: &CommandTypeId,
		object_id: Result<GameObjectId, CommandContextError>,
		field_id: Result<FieldId, CommandContextError>,
		input: &mut Cursor<&[u8]>,
	) -> Result<C2SCommand, CommandDecodeError> {
		Ok(match *command_type_id {
			CommandTypeId::ATTACH_TO_ROOM => C2SCommand::AttachToRoom,
			CommandTypeId::DETACH_FROM_ROOM => C2SCommand::DetachFromRoom,
			CommandTypeId::CREATED_GAME_OBJECT => C2SCommand::CreatedGameObject(C2SCreatedGameObjectCommand::decode(object_id?, input)?),
			CommandTypeId::DELETE => C2SCommand::Delete(DeleteGameObjectCommand { object_id: object_id? }),
			CommandTypeId::CREATE_GAME_OBJECT => C2SCommand::CreateGameObject(CreateGameObjectCommand::decode(object_id?, input)?),
			CommandTypeId::INCREMENT_LONG => C2SCommand::IncrementLongValue(IncrementLongC2SCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::INCREMENT_DOUBLE => C2SCommand::IncrementDouble(IncrementDoubleC2SCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::COMPARE_AND_SET_LONG => C2SCommand::CompareAndSetLong(CompareAndSetLongCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::COMPARE_AND_SET_STRUCTURE => {
				C2SCommand::CompareAndSetStructure(CompareAndSetStructureCommand::decode(object_id?, field_id?, input)?)
			}
			CommandTypeId::SET_DOUBLE => C2SCommand::SetField(SetFieldCommand::decode::<f64>(object_id?, field_id?, input)?),
			CommandTypeId::SET_LONG => C2SCommand::SetField(SetFieldCommand::decode::<i64>(object_id?, field_id?, input)?),
			CommandTypeId::SET_STRUCTURE => C2SCommand::SetField(SetFieldCommand::decode::<Vec<u8>>(object_id?, field_id?, input)?),
			CommandTypeId::EVENT => C2SCommand::Event(EventCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::TARGET_EVENT => C2SCommand::TargetEvent(TargetEventCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::DELETE_FIELD => C2SCommand::DeleteField(DeleteFieldCommand::decode(object_id?, field_id?, input)?),
			_ => return Err(CommandDecodeError::UnknownTypeId(*command_type_id)),
		})
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::commands::binary_value::BinaryValue;
	use crate::commands::c2s::C2SCommand;
	use crate::commands::field::FieldId;
	use crate::commands::types::create::{C2SCreatedGameObjectCommand, CreateGameObjectCommand};
	use crate::commands::types::delete::DeleteGameObjectCommand;
	use crate::commands::types::event::{EventCommand, TargetEventCommand};
	use crate::commands::types::field::SetFieldCommand;
	use crate::commands::types::float::IncrementDoubleC2SCommand;
	use crate::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand};
	use crate::commands::types::structure::CompareAndSetStructureCommand;
	use crate::commands::CommandTypeId;
	use crate::protocol::codec::commands::context::CommandContextError;
	use crate::room::access::AccessGroups;
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	#[test]
	fn should_decode_encode_attach() {
		check(C2SCommand::AttachToRoom, CommandTypeId::ATTACH_TO_ROOM, None, None);
	}
	#[test]
	fn should_decode_encode_detach() {
		check(C2SCommand::DetachFromRoom, CommandTypeId::DETACH_FROM_ROOM, None, None);
	}

	#[test]
	fn should_decode_encode_create_member_object() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			C2SCommand::CreateGameObject(CreateGameObjectCommand {
				object_id: object_id.clone(),
				template: 3,
				access_groups: AccessGroups(5),
			}),
			CommandTypeId::CREATE_GAME_OBJECT,
			Some(object_id),
			None,
		);
	}

	#[test]
	fn should_decode_encode_created() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			C2SCommand::CreatedGameObject(C2SCreatedGameObjectCommand {
				object_id: object_id.clone(),
				room_owner: false,
				singleton_key: None,
			}),
			CommandTypeId::CREATED_GAME_OBJECT,
			Some(object_id),
			None,
		);
	}

	#[test]
	fn should_decode_encode_set_long() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			C2SCommand::SetField(SetFieldCommand {
				object_id: object_id.clone(),
				field_id,
				value: 100.into(),
			}),
			CommandTypeId::SET_LONG,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_increment_long_value() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			C2SCommand::IncrementLongValue(IncrementLongC2SCommand {
				object_id: object_id.clone(),
				field_id,
				increment: 100,
			}),
			CommandTypeId::INCREMENT_LONG,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_compare_and_set_long_value() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			C2SCommand::CompareAndSetLong(CompareAndSetLongCommand {
				object_id: object_id.clone(),
				field_id,
				current: 100,
				new: 101,
				reset: Some(102),
			}),
			CommandTypeId::COMPARE_AND_SET_LONG,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_compare_and_set_structure() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			C2SCommand::CompareAndSetStructure(CompareAndSetStructureCommand {
				object_id: object_id.clone(),
				field_id,
				current: vec![100].as_slice().into(),
				new: vec![101].as_slice().into(),
				reset: Some(vec![102].as_slice().into()),
			}),
			CommandTypeId::COMPARE_AND_SET_STRUCTURE,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_double() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			C2SCommand::SetField(SetFieldCommand {
				object_id: object_id.clone(),
				field_id,
				value: 3.15.into(),
			}),
			CommandTypeId::SET_DOUBLE,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_incremental_double() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			C2SCommand::IncrementDouble(IncrementDoubleC2SCommand {
				object_id: object_id.clone(),
				field_id,
				increment: 3.15,
			}),
			CommandTypeId::INCREMENT_DOUBLE,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_structure() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			C2SCommand::SetField(SetFieldCommand {
				object_id: object_id.clone(),
				field_id,
				value: vec![1, 2, 3, 4].into(),
			}),
			CommandTypeId::SET_STRUCTURE,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_event() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			C2SCommand::Event(EventCommand {
				object_id: object_id.clone(),
				field_id,
				event: BinaryValue::from(vec![1, 2, 3, 4].as_slice()),
			}),
			CommandTypeId::EVENT,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_target_event() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			C2SCommand::TargetEvent(TargetEventCommand {
				target: 10,
				event: EventCommand {
					object_id: object_id.clone(),
					field_id,
					event: BinaryValue::from(vec![1, 2, 3, 4].as_slice()),
				},
			}),
			CommandTypeId::TARGET_EVENT,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_delete() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			C2SCommand::Delete(DeleteGameObjectCommand {
				object_id: object_id.clone(),
			}),
			CommandTypeId::DELETE,
			Some(object_id),
			None,
		);
	}

	fn check(expected: C2SCommand, command_type_id: CommandTypeId, object_id: Option<GameObjectId>, field_id: Option<FieldId>) {
		let object_id = object_id.ok_or(CommandContextError::ContextNotContainsObjectId);
		let field_id = field_id.ok_or(CommandContextError::ContextNotContainsFieldId);
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		expected.encode(&mut cursor).unwrap();
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = C2SCommand::decode(&command_type_id, object_id, field_id, &mut read_cursor).unwrap();
		assert_eq!(write_position, read_cursor.position());
		assert_eq!(expected, actual);
	}
}
