use std::io::Cursor;

use strum_macros::AsRefStr;
use thiserror::Error;

use crate::commands::s2c::S2CCommand::SetStruct;
use crate::commands::types::event::EventCommand;
use crate::commands::types::float::SetFloat64Command;
use crate::commands::types::load::{CreateGameObjectCommand, CreatedGameObjectCommand};
use crate::commands::types::long::SetLongCommand;
use crate::commands::types::structure::StructureCommand;
use crate::commands::types::unload::DeleteGameObjectCommand;
use crate::commands::{CommandTypeId, FieldType};
use crate::constants::FieldId;
use crate::protocol::codec::commands::context::{CommandContext, CommandContextError};
use crate::room::object::GameObjectId;
use crate::room::RoomMemberId;

#[derive(Debug, PartialEq, Clone, AsRefStr)]
pub enum S2CCommand {
	Create(CreateGameObjectCommand),
	Created(CreatedGameObjectCommand),
	SetLong(SetLongCommand),
	SetFloat(SetFloat64Command),
	SetStruct(StructureCommand),
	Event(EventCommand),
	Delete(DeleteGameObjectCommand),
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
			S2CCommand::SetFloat(command) => Some(command.field_id),
			S2CCommand::SetStruct(command) => Some(command.field_id),
			S2CCommand::Event(command) => Some(command.field_id),
			S2CCommand::Delete(_) => Option::None,
		}
	}

	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			S2CCommand::Create(command) => Some(command.object_id.clone()),
			S2CCommand::Created(command) => Some(command.object_id.clone()),
			S2CCommand::SetLong(command) => Some(command.object_id.clone()),
			S2CCommand::SetFloat(command) => Some(command.object_id.clone()),
			S2CCommand::SetStruct(command) => Some(command.object_id.clone()),
			S2CCommand::Event(command) => Some(command.object_id.clone()),
			S2CCommand::Delete(command) => Some(command.object_id.clone()),
		}
	}

	pub fn get_field_type(&self) -> Option<FieldType> {
		match self {
			S2CCommand::Create(_) => Option::None,
			S2CCommand::Created(_) => Option::None,
			S2CCommand::SetLong(_) => Option::Some(FieldType::Long),
			S2CCommand::SetFloat(_) => Option::Some(FieldType::Double),
			S2CCommand::SetStruct(_) => Option::Some(FieldType::Structure),
			S2CCommand::Event(_) => Option::Some(FieldType::Event),
			S2CCommand::Delete(_) => Option::None,
		}
	}

	pub fn get_type_id(&self) -> CommandTypeId {
		match self {
			S2CCommand::Create(_) => CommandTypeId(0),
			S2CCommand::Created(_) => CommandTypeId(1),
			S2CCommand::SetLong(_) => CommandTypeId(2),
			S2CCommand::SetFloat(_) => CommandTypeId(3),
			S2CCommand::SetStruct(_) => CommandTypeId(4),
			S2CCommand::Event(_) => CommandTypeId(5),
			S2CCommand::Delete(_) => CommandTypeId(6),
		}
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			S2CCommand::Create(command) => command.encode(out),
			S2CCommand::Created(_) => Ok(()),
			S2CCommand::SetLong(command) => command.encode(out),
			S2CCommand::SetFloat(command) => command.encode(out),
			S2CCommand::SetStruct(command) => command.encode(out),
			S2CCommand::Event(command) => command.encode(out),
			S2CCommand::Delete(_) => Ok(()),
		}
	}

	pub fn decode(
		command_type_id: CommandTypeId,
		context: &CommandContext,
		input: &mut Cursor<&mut [u8]>,
	) -> Result<S2CCommand, S2CCommandDecodeError> {
		let object_id = context.get_object_id()?.clone();
		match command_type_id {
			CommandTypeId(0) => return Ok(S2CCommand::Create(CreateGameObjectCommand::decode(object_id, input)?)),
			CommandTypeId(1) => return Ok(S2CCommand::Created(CreatedGameObjectCommand { object_id })),
			CommandTypeId(6) => return Ok(S2CCommand::Delete(DeleteGameObjectCommand { object_id })),
			_ => {}
		};
		let field_id = context.get_field_id()?;
		Ok(match command_type_id {
			CommandTypeId(2) => S2CCommand::SetLong(SetLongCommand::decode(object_id, field_id, input)?),
			CommandTypeId(3) => S2CCommand::SetFloat(SetFloat64Command::decode(object_id, field_id, input)?),
			CommandTypeId(4) => S2CCommand::SetStruct(StructureCommand::decode(object_id, field_id, input)?),
			CommandTypeId(5) => S2CCommand::Event(EventCommand::decode(object_id, field_id, input)?),
			_ => Err(S2CCommandDecodeError::UnknownTypeId(command_type_id))?,
		})
	}
}

#[derive(Error, Debug)]
pub enum S2CCommandDecodeError {
	#[error("Unknown type {:?}.",.0)]
	UnknownTypeId(CommandTypeId),
	#[error("IO error {:?}",.source)]
	Io {
		#[from]
		source: std::io::Error,
	},
	#[error("CommandContext error {:?}", .source)]
	CommandContext {
		#[from]
		source: CommandContextError,
	},
}
