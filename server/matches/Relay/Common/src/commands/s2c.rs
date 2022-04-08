use std::io::Cursor;

use strum_macros::AsRefStr;

use crate::commands::types::event::EventCommand;
use crate::commands::types::field::DeleteFieldCommand;
use crate::commands::types::float::SetDoubleCommand;
use crate::commands::types::load::{CreatedGameObjectCommand, S2CCreateGameObjectCommand};
use crate::commands::types::long::SetLongCommand;
use crate::commands::types::structure::SetStructureCommand;
use crate::commands::types::unload::DeleteGameObjectCommand;
use crate::commands::{CommandDecodeError, CommandTypeId, FieldType};
use crate::constants::FieldId;
use crate::protocol::codec::commands::context::CommandContextError;
use crate::room::object::GameObjectId;
use crate::room::RoomMemberId;

#[derive(Debug, PartialEq, Clone, AsRefStr)]
pub enum S2CCommand {
	Create(S2CCreateGameObjectCommand),
	Created(CreatedGameObjectCommand),
	SetLong(SetLongCommand),
	SetDouble(SetDoubleCommand),
	SetStructure(SetStructureCommand),
	Event(EventCommand),
	Delete(DeleteGameObjectCommand),
	DeleteField(DeleteFieldCommand),
}

#[derive(Debug, PartialEq, Clone)]
pub struct S2CCommandWithCreator {
	pub command: S2CCommand,
	pub creator: RoomMemberId,
}
impl S2CCommand {
	pub fn get_field_id(&self) -> Option<FieldId> {
		match self {
			S2CCommand::Create(_) => Option::None,
			S2CCommand::Created(_) => Option::None,
			S2CCommand::SetLong(command) => Some(command.field_id),
			S2CCommand::SetDouble(command) => Some(command.field_id),
			S2CCommand::SetStructure(command) => Some(command.field_id),
			S2CCommand::Event(command) => Some(command.field_id),
			S2CCommand::Delete(_) => Option::None,
			S2CCommand::DeleteField(command) => Some(command.field_id),
		}
	}

	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			S2CCommand::Create(command) => Some(command.object_id.clone()),
			S2CCommand::Created(command) => Some(command.object_id.clone()),
			S2CCommand::SetLong(command) => Some(command.object_id.clone()),
			S2CCommand::SetDouble(command) => Some(command.object_id.clone()),
			S2CCommand::SetStructure(command) => Some(command.object_id.clone()),
			S2CCommand::Event(command) => Some(command.object_id.clone()),
			S2CCommand::Delete(command) => Some(command.object_id.clone()),
			S2CCommand::DeleteField(command) => Some(command.object_id.clone()),
		}
	}

	pub fn get_field_type(&self) -> Option<FieldType> {
		match self {
			S2CCommand::Create(_) => Option::None,
			S2CCommand::Created(_) => Option::None,
			S2CCommand::SetLong(_) => Option::Some(FieldType::Long),
			S2CCommand::SetDouble(_) => Option::Some(FieldType::Double),
			S2CCommand::SetStructure(_) => Option::Some(FieldType::Structure),
			S2CCommand::Event(_) => Option::Some(FieldType::Event),
			S2CCommand::Delete(_) => Option::None,
			S2CCommand::DeleteField(command) => Some(command.field_type.clone()),
		}
	}

	pub fn get_type_id(&self) -> CommandTypeId {
		match self {
			S2CCommand::Create(_) => CommandTypeId::CREATE,
			S2CCommand::Created(_) => CommandTypeId::CREATED,
			S2CCommand::SetLong(_) => CommandTypeId::SET_LONG,
			S2CCommand::SetDouble(_) => CommandTypeId::SET_DOUBLE,
			S2CCommand::SetStructure(_) => CommandTypeId::SET_STRUCTURE,
			S2CCommand::Event(_) => CommandTypeId::EVENT,
			S2CCommand::Delete(_) => CommandTypeId::DELETE,
			S2CCommand::DeleteField(_) => CommandTypeId::DELETE_FIELD,
		}
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			S2CCommand::Create(command) => command.encode(out),
			S2CCommand::Created(_) => Ok(()),
			S2CCommand::SetLong(command) => command.encode(out),
			S2CCommand::SetDouble(command) => command.encode(out),
			S2CCommand::SetStructure(command) => command.encode(out),
			S2CCommand::Event(command) => command.encode(out),
			S2CCommand::Delete(_) => Ok(()),
			S2CCommand::DeleteField(command) => command.encode(out),
		}
	}

	pub fn decode(
		command_type_id: &CommandTypeId,
		object_id: Result<GameObjectId, CommandContextError>,
		field_id: Result<FieldId, CommandContextError>,
		input: &mut Cursor<&[u8]>,
	) -> Result<S2CCommand, CommandDecodeError> {
		Ok(match *command_type_id {
			CommandTypeId::CREATE => S2CCommand::Create(S2CCreateGameObjectCommand::decode(object_id?, input)?),
			CommandTypeId::CREATED => S2CCommand::Created(CreatedGameObjectCommand { object_id: object_id? }),
			CommandTypeId::DELETE => S2CCommand::Delete(DeleteGameObjectCommand { object_id: object_id? }),
			CommandTypeId::SET_LONG => S2CCommand::SetLong(SetLongCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::SET_DOUBLE => S2CCommand::SetDouble(SetDoubleCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::SET_STRUCTURE => S2CCommand::SetStructure(SetStructureCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::EVENT => S2CCommand::Event(EventCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::DELETE_FIELD => S2CCommand::DeleteField(DeleteFieldCommand::decode(field_id?, object_id?, input)?),
			_ => return Err(CommandDecodeError::UnknownTypeId(*command_type_id)),
		})
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::commands::types::load::S2CCreateGameObjectCommand;
	use crate::{
		commands::s2c::S2CCommand,
		commands::types::event::EventCommand,
		commands::types::float::SetDoubleCommand,
		commands::types::load::CreatedGameObjectCommand,
		commands::types::long::SetLongCommand,
		commands::types::structure::SetStructureCommand,
		commands::types::unload::DeleteGameObjectCommand,
		commands::{CommandBuffer, CommandTypeId},
		constants::FieldId,
		protocol::codec::commands::context::CommandContextError,
		room::access::AccessGroups,
		room::object::GameObjectId,
		room::owner::GameObjectOwner,
	};

	#[test]
	fn should_decode_encode_create() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			S2CCommand::Create(S2CCreateGameObjectCommand {
				object_id: object_id.clone(),
				template: 3,
				access_groups: AccessGroups(5),
			}),
			CommandTypeId::CREATE,
			Some(object_id),
			None,
		);
	}

	#[test]
	fn should_decode_encode_created() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			S2CCommand::Created(CreatedGameObjectCommand {
				object_id: object_id.clone(),
			}),
			CommandTypeId::CREATED,
			Some(object_id),
			None,
		);
	}

	#[test]
	fn should_decode_encode_set_long() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			S2CCommand::SetLong(SetLongCommand {
				object_id: object_id.clone(),
				field_id,
				value: 100,
			}),
			CommandTypeId::SET_LONG,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_double() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			S2CCommand::SetDouble(SetDoubleCommand {
				object_id: object_id.clone(),
				field_id,
				value: 3.15,
			}),
			CommandTypeId::SET_DOUBLE,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_structure() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			S2CCommand::SetStructure(SetStructureCommand {
				object_id: object_id.clone(),
				field_id,
				structure: CommandBuffer::from_slice(vec![1, 2, 3, 4].as_slice()).unwrap(),
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
			S2CCommand::Event(EventCommand {
				object_id: object_id.clone(),
				field_id,
				event: CommandBuffer::from_slice(vec![1, 2, 3, 4].as_slice()).unwrap(),
			}),
			CommandTypeId::EVENT,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_delete() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			S2CCommand::Delete(DeleteGameObjectCommand {
				object_id: object_id.clone(),
			}),
			CommandTypeId::DELETE,
			Some(object_id),
			None,
		);
	}

	fn check(excepted: S2CCommand, command_type_id: CommandTypeId, object_id: Option<GameObjectId>, field_id: Option<FieldId>) {
		let object_id = object_id.ok_or(CommandContextError::ContextNotContainsObjectId);
		let field_id = field_id.ok_or(CommandContextError::ContextNotContainsFieldId);
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		excepted.encode(&mut cursor).unwrap();
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = S2CCommand::decode(&command_type_id, object_id, field_id, &mut read_cursor).unwrap();

		assert_eq!(write_position, read_cursor.position());
		assert_eq!(excepted, actual);
	}
}
