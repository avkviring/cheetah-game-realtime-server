use std::io::Cursor;

use crate::commands::field::FieldId;
use crate::commands::field_value::FieldValue;
use crate::commands::types::create::{CreateGameObjectCommand, GameObjectCreatedS2CCommand};
use crate::commands::types::delete::DeleteGameObjectCommand;
use crate::commands::types::event::EventCommand;
use crate::commands::types::field::{DeleteFieldCommand, SetFieldCommand};
use crate::commands::types::forwarded::ForwardedCommand;
use crate::commands::{CommandDecodeError, CommandTypeId, FieldType};
use crate::protocol::codec::commands::context::CommandContextError;
use crate::room::object::GameObjectId;
use crate::room::RoomMemberId;
use strum_macros::AsRefStr;

#[derive(Debug, PartialEq, Clone, AsRefStr)]
#[allow(clippy::large_enum_variant)]
pub enum S2CCommand {
	Create(CreateGameObjectCommand),
	Created(GameObjectCreatedS2CCommand),
	SetField(SetFieldCommand),
	Event(EventCommand),
	Delete(DeleteGameObjectCommand),
	DeleteField(DeleteFieldCommand),
	Forwarded(Box<ForwardedCommand>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct S2CCommandWithCreator {
	pub command: S2CCommand,
	pub creator: RoomMemberId,
}

impl S2CCommand {
	pub fn new_set_command(value: FieldValue, object_id: GameObjectId, field_id: u16) -> S2CCommand {
		S2CCommand::SetField(SetFieldCommand { field_id, object_id, value })
	}

	pub fn get_field_id(&self) -> Option<FieldId> {
		match self {
			S2CCommand::Create(_) => None,
			S2CCommand::Created(_) => None,
			S2CCommand::SetField(command) => Some(command.field_id),
			S2CCommand::Event(command) => Some(command.field_id),
			S2CCommand::Delete(_) => None,
			S2CCommand::DeleteField(command) => Some(command.field_id),
			S2CCommand::Forwarded(command) => command.c2s.get_field_id(),
		}
	}

	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			S2CCommand::Create(command) => Some(command.object_id.clone()),
			S2CCommand::Created(command) => Some(command.object_id.clone()),
			S2CCommand::SetField(command) => Some(command.object_id.clone()),
			S2CCommand::Event(command) => Some(command.object_id.clone()),
			S2CCommand::Delete(command) => Some(command.object_id.clone()),
			S2CCommand::DeleteField(command) => Some(command.object_id.clone()),
			S2CCommand::Forwarded(command) => command.c2s.get_object_id(),
		}
	}

	pub fn get_field_type(&self) -> Option<FieldType> {
		match self {
			S2CCommand::Create(_) => None,
			S2CCommand::Created(_) => None,
			S2CCommand::SetField(command) => Some(command.value.field_type()),
			S2CCommand::Event(_) => Some(FieldType::Event),
			S2CCommand::Delete(_) => None,
			S2CCommand::DeleteField(command) => Some(command.field_type),
			S2CCommand::Forwarded(command) => command.c2s.get_field_type(),
		}
	}

	pub fn get_type_id(&self) -> CommandTypeId {
		match self {
			S2CCommand::Create(_) => CommandTypeId::CreateGameObject,
			S2CCommand::Created(_) => CommandTypeId::CreatedGameObject,
			S2CCommand::SetField(command) => match command.value {
				FieldValue::Long(_) => CommandTypeId::SetLong,
				FieldValue::Double(_) => CommandTypeId::SetDouble,
				FieldValue::Structure(_) => CommandTypeId::SetStructure,
			},
			S2CCommand::Event(_) => CommandTypeId::Event,
			S2CCommand::Delete(_) => CommandTypeId::Delete,
			S2CCommand::DeleteField(_) => CommandTypeId::DeleteField,
			S2CCommand::Forwarded(_) => CommandTypeId::Forwarded,
		}
	}

	pub fn get_trace_string(&self) -> String {
		match self {
			S2CCommand::Create(command) => format!("access({:?}), template({:?}) ", command.access_groups.0, command.template),
			S2CCommand::Created(_) => "".to_string(),
			S2CCommand::SetField(command) => format!("{:?}", command.value),
			S2CCommand::Event(command) => format!("{:?}", command.event),
			S2CCommand::Delete(_) => "".to_string(),
			S2CCommand::DeleteField(_) => "".to_string(),
			S2CCommand::Forwarded(command) => format!("forward: user({:?}) command({:?})", command.creator, command.c2s.get_trace_string()),
		}
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			S2CCommand::Create(command) => command.encode(out),
			S2CCommand::Created(_) => Ok(()),
			S2CCommand::SetField(command) => command.encode(out),
			S2CCommand::Event(command) => command.encode(out),
			S2CCommand::Delete(_) => Ok(()),
			S2CCommand::DeleteField(command) => command.encode(out),
			S2CCommand::Forwarded(command) => command.encode(out),
		}
	}

	pub fn decode(
		command_type_id: &CommandTypeId,
		object_id: Result<GameObjectId, CommandContextError>,
		field_id: Result<FieldId, CommandContextError>,
		input: &mut Cursor<&[u8]>,
	) -> Result<S2CCommand, CommandDecodeError> {
		Ok(match *command_type_id {
			CommandTypeId::CreateGameObject => S2CCommand::Create(CreateGameObjectCommand::decode(object_id?, input)?),
			CommandTypeId::CreatedGameObject => S2CCommand::Created(GameObjectCreatedS2CCommand { object_id: object_id? }),
			CommandTypeId::Delete => S2CCommand::Delete(DeleteGameObjectCommand { object_id: object_id? }),
			CommandTypeId::SetLong => S2CCommand::SetField(SetFieldCommand::decode::<i64>(object_id?, field_id?, input)?),
			CommandTypeId::SetDouble => S2CCommand::SetField(SetFieldCommand::decode::<f64>(object_id?, field_id?, input)?),
			CommandTypeId::SetStructure => S2CCommand::SetField(SetFieldCommand::decode::<Vec<u8>>(object_id?, field_id?, input)?),
			CommandTypeId::Event => S2CCommand::Event(EventCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::DeleteField => S2CCommand::DeleteField(DeleteFieldCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::Forwarded => S2CCommand::Forwarded(Box::new(ForwardedCommand::decode(object_id, field_id, input)?)),
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
	use crate::commands::types::create::{CreateGameObjectCommand, GameObjectCreatedS2CCommand};
	use crate::commands::types::delete::DeleteGameObjectCommand;
	use crate::commands::types::event::TargetEventCommand;
	use crate::commands::types::field::SetFieldCommand;
	use crate::commands::types::forwarded::ForwardedCommand;
	use crate::commands::CommandTypeId;
	use crate::{
		commands::s2c::S2CCommand, commands::types::event::EventCommand, protocol::codec::commands::context::CommandContextError,
		room::access::AccessGroups, room::object::GameObjectId, room::owner::GameObjectOwner,
	};

	#[test]
	fn should_decode_encode_create() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			S2CCommand::Create(CreateGameObjectCommand {
				object_id: object_id.clone(),
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
			S2CCommand::Created(GameObjectCreatedS2CCommand {
				object_id: object_id.clone(),
			}),
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
			S2CCommand::SetField(SetFieldCommand {
				object_id: object_id.clone(),
				field_id,
				value: 100.into(),
			}),
			CommandTypeId::SetLong,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_double() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			S2CCommand::SetField(SetFieldCommand {
				object_id: object_id.clone(),
				field_id,
				value: 3.15.into(),
			}),
			CommandTypeId::SetDouble,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_structure() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			S2CCommand::SetField(SetFieldCommand {
				object_id: object_id.clone(),
				field_id,
				value: vec![1, 2, 3, 4].into(),
			}),
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
			S2CCommand::Event(EventCommand {
				object_id: object_id.clone(),
				field_id,
				event: BinaryValue::from(vec![1, 2, 3, 4].as_slice()),
			}),
			CommandTypeId::Event,
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
			CommandTypeId::Delete,
			Some(object_id),
			None,
		);
	}

	#[test]
	fn should_decode_encode_forwarded() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			S2CCommand::Forwarded(Box::new(ForwardedCommand {
				creator: 123,
				c2s: C2SCommand::TargetEvent(TargetEventCommand {
					target: 10,
					event: EventCommand {
						object_id: object_id.clone(),
						field_id,
						event: BinaryValue::from(vec![1, 2, 3, 4].as_slice()),
					},
				}),
			})),
			CommandTypeId::Forwarded,
			Some(object_id),
			Some(field_id),
		);
	}

	fn check(expected: S2CCommand, command_type_id: CommandTypeId, object_id: Option<GameObjectId>, field_id: Option<FieldId>) {
		let object_id = object_id.ok_or(CommandContextError::ContextNotContainsObjectId);
		let field_id = field_id.ok_or(CommandContextError::ContextNotContainsFieldId);
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		expected.encode(&mut cursor).unwrap();
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = S2CCommand::decode(&command_type_id, object_id, field_id, &mut read_cursor).unwrap();

		assert_eq!(write_position, read_cursor.position());
		assert_eq!(expected, actual);
	}
}
