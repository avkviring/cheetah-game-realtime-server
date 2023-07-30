use std::io::Cursor;

use strum_macros::AsRefStr;

use cheetah_protocol::RoomMemberId;

use crate::commands::context::CommandContextError;
use crate::commands::types::create::{CreateGameObjectCommand, GameObjectCreatedS2CCommand};
use crate::commands::types::delete::DeleteGameObjectCommand;
use crate::commands::types::event::EventCommand;
use crate::commands::types::field::DeleteFieldCommand;
use crate::commands::types::float::SetDoubleCommand;
use crate::commands::types::forwarded::ForwardedCommand;
use crate::commands::types::long::SetLongCommand;
use crate::commands::types::member::{MemberConnected, MemberDisconnected};
use crate::commands::types::structure::SetStructureCommand;
use crate::commands::{CommandDecodeError, CommandTypeId};
use crate::room::field::{Field, FieldId, FieldType};
use crate::room::object::GameObjectId;

#[derive(Debug, PartialEq, Clone, AsRefStr)]
#[allow(clippy::large_enum_variant)]
pub enum S2CCommand {
	Create(CreateGameObjectCommand),
	Created(GameObjectCreatedS2CCommand),
	SetLong(SetLongCommand),
	SetDouble(SetDoubleCommand),
	SetStructure(Box<SetStructureCommand>),
	Event(Box<EventCommand>),
	Delete(DeleteGameObjectCommand),
	DeleteField(DeleteFieldCommand),
	Forwarded(Box<ForwardedCommand>),
	MemberConnected(MemberConnected),
	MemberDisconnected(MemberDisconnected),
}

#[derive(Debug, PartialEq, Clone)]
pub struct S2CCommandWithCreator {
	pub command: S2CCommand,
	pub creator: RoomMemberId,
}

#[derive(Debug, Clone)]
pub struct S2CCommandWithMeta {
	pub field: Option<Field>,
	pub creator: RoomMemberId,
	pub command: S2CCommand,
}

impl S2CCommand {
	#[must_use]
	pub fn get_field_id(&self) -> Option<FieldId> {
		match self {
			S2CCommand::Create(_) => None,
			S2CCommand::Created(_) => None,
			S2CCommand::Event(command) => Some(command.field_id),
			S2CCommand::Delete(_) => None,
			S2CCommand::DeleteField(command) => Some(command.field_id),
			S2CCommand::Forwarded(command) => command.c2s.get_field_id(),
			S2CCommand::SetLong(command) => command.field_id.into(),
			S2CCommand::SetDouble(command) => command.field_id.into(),
			S2CCommand::SetStructure(command) => command.field_id.into(),
			S2CCommand::MemberConnected(_) => None,
			S2CCommand::MemberDisconnected(_) => None,
		}
	}

	#[must_use]
	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			S2CCommand::Create(command) => Some(command.object_id),
			S2CCommand::Created(command) => Some(command.object_id),
			S2CCommand::Event(command) => Some(command.object_id),
			S2CCommand::Delete(command) => Some(command.object_id),
			S2CCommand::DeleteField(command) => Some(command.object_id),
			S2CCommand::Forwarded(command) => command.c2s.get_object_id(),
			S2CCommand::SetLong(command) => command.object_id.into(),
			S2CCommand::SetDouble(command) => command.object_id.into(),
			S2CCommand::SetStructure(command) => command.object_id.into(),
			S2CCommand::MemberConnected(_) => None,
			S2CCommand::MemberDisconnected(_) => None,
		}
	}

	#[must_use]
	pub fn get_field_type(&self) -> Option<FieldType> {
		match self {
			S2CCommand::Create(_) => None,
			S2CCommand::Created(_) => None,
			S2CCommand::Event(_) => Some(FieldType::Event),
			S2CCommand::Delete(_) => None,
			S2CCommand::DeleteField(command) => Some(command.field_type),
			S2CCommand::Forwarded(command) => command.c2s.get_field_type(),
			S2CCommand::SetLong(_) => FieldType::Long.into(),
			S2CCommand::SetDouble(_) => FieldType::Double.into(),
			S2CCommand::SetStructure(_) => FieldType::Structure.into(),
			S2CCommand::MemberConnected(_) => None,
			S2CCommand::MemberDisconnected(_) => None,
		}
	}

	#[must_use]
	pub fn get_type_id(&self) -> CommandTypeId {
		match self {
			S2CCommand::Create(_) => CommandTypeId::CreateGameObject,
			S2CCommand::Created(_) => CommandTypeId::CreatedGameObject,
			S2CCommand::Event(_) => CommandTypeId::SendEvent,
			S2CCommand::Delete(_) => CommandTypeId::DeleteObject,
			S2CCommand::DeleteField(_) => CommandTypeId::DeleteField,
			S2CCommand::Forwarded(_) => CommandTypeId::Forwarded,
			S2CCommand::SetLong(_) => CommandTypeId::SetLong,
			S2CCommand::SetDouble(_) => CommandTypeId::SetDouble,
			S2CCommand::SetStructure(_) => CommandTypeId::SetStructure,
			S2CCommand::MemberConnected(_) => CommandTypeId::MemberConnected,
			S2CCommand::MemberDisconnected(_) => CommandTypeId::MemberDisconnected,
		}
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			S2CCommand::Create(command) => command.encode(out),
			S2CCommand::Created(_) => Ok(()),
			S2CCommand::Event(command) => command.encode(out),
			S2CCommand::Delete(_) => Ok(()),
			S2CCommand::DeleteField(command) => command.encode(out),
			S2CCommand::Forwarded(command) => command.encode(out),
			S2CCommand::MemberConnected(command) => command.encode(out),
			S2CCommand::SetLong(command) => command.encode(out),
			S2CCommand::SetDouble(command) => command.encode(out),
			S2CCommand::SetStructure(command) => command.encode(out),
			S2CCommand::MemberDisconnected(command) => command.encode(out),
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
			CommandTypeId::DeleteObject => S2CCommand::Delete(DeleteGameObjectCommand { object_id: object_id? }),
			CommandTypeId::SetLong => S2CCommand::SetLong(SetLongCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::SetDouble => S2CCommand::SetDouble(SetDoubleCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::SetStructure => S2CCommand::SetStructure(SetStructureCommand::decode(object_id?, field_id?, input)?.into()),
			CommandTypeId::SendEvent => S2CCommand::Event(EventCommand::decode(object_id?, field_id?, input)?.into()),
			CommandTypeId::DeleteField => S2CCommand::DeleteField(DeleteFieldCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::Forwarded => S2CCommand::Forwarded(Box::new(ForwardedCommand::decode(object_id, field_id, input)?)),
			CommandTypeId::MemberConnected => S2CCommand::MemberConnected(MemberConnected::decode(input)?),
			CommandTypeId::MemberDisconnected => S2CCommand::MemberDisconnected(MemberDisconnected::decode(input)?),
			_ => return Err(CommandDecodeError::UnknownTypeId(*command_type_id)),
		})
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::commands::c2s::C2SCommand;
	use crate::commands::context::CommandContextError;
	use crate::commands::s2c::S2CCommand;
	use crate::commands::types::create::{CreateGameObjectCommand, GameObjectCreatedS2CCommand};
	use crate::commands::types::delete::DeleteGameObjectCommand;
	use crate::commands::types::event::{EventCommand, TargetEventCommand};
	use crate::commands::types::float::SetDoubleCommand;
	use crate::commands::types::forwarded::ForwardedCommand;
	use crate::commands::types::long::SetLongCommand;
	use crate::commands::types::member::MemberConnected;
	use crate::commands::types::structure::SetStructureCommand;
	use crate::commands::CommandTypeId;
	use crate::room::access::AccessGroups;
	use crate::room::buffer::Buffer;
	use crate::room::field::FieldId;
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	#[test]
	fn should_decode_encode_create() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			&S2CCommand::Create(CreateGameObjectCommand {
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
		check(&S2CCommand::Created(GameObjectCreatedS2CCommand { object_id }), CommandTypeId::CreatedGameObject, Some(object_id), None);
	}

	#[test]
	fn should_decode_encode_set_long() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&S2CCommand::SetLong(SetLongCommand { object_id, field_id, value: 100 }),
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
			&S2CCommand::SetDouble(SetDoubleCommand { object_id, field_id, value: 3.15 }),
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
			&S2CCommand::SetStructure(
				SetStructureCommand {
					object_id,
					field_id,
					value: Buffer::from([1, 2, 3, 4].as_ref()).into(),
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
			&S2CCommand::Event(
				EventCommand {
					object_id,
					field_id,
					event: Buffer::from(vec![1, 2, 3, 4].as_slice()),
				}
				.into(),
			),
			CommandTypeId::SendEvent,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_delete() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(&S2CCommand::Delete(DeleteGameObjectCommand { object_id }), CommandTypeId::DeleteObject, Some(object_id), None);
	}

	#[test]
	fn should_decode_encode_forwarded() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&S2CCommand::Forwarded(Box::new(ForwardedCommand {
				creator: 123,
				c2s: C2SCommand::TargetEvent(
					TargetEventCommand {
						target: 10,
						event: EventCommand {
							object_id,
							field_id,
							event: Buffer::from(vec![1, 2, 3, 4].as_slice()),
						},
					}
					.into(),
				),
			})),
			CommandTypeId::Forwarded,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_member_connected() {
		check(&S2CCommand::MemberConnected(MemberConnected { member_id: 100 }), CommandTypeId::MemberConnected, None, None);
	}

	fn check(expected: &S2CCommand, command_type_id: CommandTypeId, object_id: Option<GameObjectId>, field_id: Option<FieldId>) {
		let object_id = object_id.ok_or(CommandContextError::ContextNotContainsObjectId);
		let field_id = field_id.ok_or(CommandContextError::ContextNotContainsFieldId);
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		expected.encode(&mut cursor).unwrap();
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = S2CCommand::decode(&command_type_id, object_id, field_id, &mut read_cursor).unwrap();

		assert_eq!(write_position, read_cursor.position());
		assert_eq!(expected, &actual);
	}
}
