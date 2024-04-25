use std::io::Cursor;

use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

use crate::commands::context::CommandContextError;
use crate::commands::types::create::{CreateGameObject, GameObjectCreated};
use crate::commands::types::field::DeleteField;
use crate::commands::types::float::DoubleField;
use crate::commands::types::long::LongField;
use crate::commands::types::member::{MemberConnected, MemberDisconnected};
use crate::commands::types::structure::BinaryField;
use crate::commands::{CommandDecodeError, CommandTypeId};
use crate::room::field::{FieldId, FieldType};
use crate::room::object::GameObjectId;

#[derive(Debug, PartialEq, Clone, AsRefStr, Serialize, Deserialize)]
pub enum S2CCommand {
	Create(CreateGameObject),
	Created(GameObjectCreated),
	SetLong(LongField),
	SetDouble(DoubleField),
	SetStructure(BinaryField),
	Event(BinaryField),
	Delete(GameObjectId),
	DeleteField(DeleteField),
	MemberConnected(MemberConnected),
	MemberDisconnected(MemberDisconnected),
	AddItem(BinaryField),
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
			S2CCommand::SetLong(command) => command.field_id.into(),
			S2CCommand::SetDouble(command) => command.field_id.into(),
			S2CCommand::SetStructure(command) => command.field_id.into(),
			S2CCommand::MemberConnected(_) => None,
			S2CCommand::MemberDisconnected(_) => None,
			S2CCommand::AddItem(command) => command.field_id.into(),
		}
	}

	#[must_use]
	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			S2CCommand::Create(command) => Some(command.object_id),
			S2CCommand::Created(command) => Some(command.object_id),
			S2CCommand::Event(command) => Some(command.object_id),
			S2CCommand::Delete(object_id) => Some(object_id.clone()),
			S2CCommand::DeleteField(command) => Some(command.object_id),
			S2CCommand::SetLong(command) => command.object_id.into(),
			S2CCommand::SetDouble(command) => command.object_id.into(),
			S2CCommand::SetStructure(command) => command.object_id.into(),
			S2CCommand::MemberConnected(_) => None,
			S2CCommand::MemberDisconnected(_) => None,
			S2CCommand::AddItem(command) => command.object_id.into(),
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
			S2CCommand::SetLong(_) => FieldType::Long.into(),
			S2CCommand::SetDouble(_) => FieldType::Double.into(),
			S2CCommand::SetStructure(_) => FieldType::Structure.into(),
			S2CCommand::MemberConnected(_) => None,
			S2CCommand::MemberDisconnected(_) => None,
			S2CCommand::AddItem(_) => FieldType::Items.into(),
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
			S2CCommand::SetLong(_) => CommandTypeId::SetLong,
			S2CCommand::SetDouble(_) => CommandTypeId::SetDouble,
			S2CCommand::SetStructure(_) => CommandTypeId::SetStructure,
			S2CCommand::MemberConnected(_) => CommandTypeId::MemberConnected,
			S2CCommand::MemberDisconnected(_) => CommandTypeId::MemberDisconnected,
			S2CCommand::AddItem(_) => CommandTypeId::AddItem,
		}
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			S2CCommand::Create(command) => command.encode(out),
			S2CCommand::Created(_) => Ok(()),
			S2CCommand::Event(command) => command.encode(out),
			S2CCommand::Delete(_) => Ok(()),
			S2CCommand::DeleteField(command) => command.encode(out),
			S2CCommand::MemberConnected(command) => command.encode(out),
			S2CCommand::SetLong(command) => command.encode(out),
			S2CCommand::SetDouble(command) => command.encode(out),
			S2CCommand::SetStructure(command) => command.encode(out),
			S2CCommand::MemberDisconnected(command) => command.encode(out),
			S2CCommand::AddItem(command) => command.encode(out),
		}
	}

	pub fn decode(
		command_type_id: &CommandTypeId,
		object_id: Result<GameObjectId, CommandContextError>,
		field_id: Result<FieldId, CommandContextError>,
		input: &mut Cursor<&[u8]>,
	) -> Result<S2CCommand, CommandDecodeError> {
		Ok(match *command_type_id {
			CommandTypeId::CreateGameObject => S2CCommand::Create(CreateGameObject::decode(object_id?, input)?),
			CommandTypeId::CreatedGameObject => S2CCommand::Created(GameObjectCreated { object_id: object_id? }),
			CommandTypeId::DeleteObject => S2CCommand::Delete(object_id?),
			CommandTypeId::SetLong => S2CCommand::SetLong(LongField::decode(object_id?, field_id?, input)?),
			CommandTypeId::SetDouble => S2CCommand::SetDouble(DoubleField::decode(object_id?, field_id?, input)?),
			CommandTypeId::SetStructure => S2CCommand::SetStructure(BinaryField::decode(object_id?, field_id?, input)?.into()),
			CommandTypeId::SendEvent => S2CCommand::Event(BinaryField::decode(object_id?, field_id?, input)?.into()),
			CommandTypeId::DeleteField => S2CCommand::DeleteField(DeleteField::decode(object_id?, field_id?, input)?),
			CommandTypeId::MemberConnected => S2CCommand::MemberConnected(MemberConnected::decode(input)?),
			CommandTypeId::MemberDisconnected => S2CCommand::MemberDisconnected(MemberDisconnected::decode(input)?),
			CommandTypeId::AddItem => S2CCommand::AddItem(BinaryField::decode(object_id?, field_id?, input)?.into()),
			_ => return Err(CommandDecodeError::UnknownTypeId(*command_type_id)),
		})
	}
}
