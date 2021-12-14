use std::io::Cursor;

use strum_macros::AsRefStr;
use thiserror::Error;

use crate::commands::types::event::{EventCommand, TargetEventCommand};
use crate::commands::types::float::{IncrementFloat64C2SCommand, SetFloat64Command};
use crate::commands::types::load::{CreateGameObjectCommand, CreatedGameObjectCommand};
use crate::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand};
use crate::commands::types::structure::StructureCommand;
use crate::commands::types::unload::DeleteGameObjectCommand;
use crate::commands::{CommandTypeId, FieldType};
use crate::constants::FieldId;
use crate::protocol::codec::commands::context::{CommandContext, CommandContextError};
use crate::room::object::GameObjectId;

#[derive(Debug, PartialEq, Clone, AsRefStr)]
pub enum C2SCommand {
	Create(CreateGameObjectCommand),
	Created(CreatedGameObjectCommand),
	SetLong(SetLongCommand),
	IncrementLongValue(IncrementLongC2SCommand),
	CompareAndSetLongValue(CompareAndSetLongCommand),
	SetFloat(SetFloat64Command),
	IncrementFloatCounter(IncrementFloat64C2SCommand),
	SetStruct(StructureCommand),
	Event(EventCommand),
	TargetEvent(TargetEventCommand),
	Delete(DeleteGameObjectCommand),
	///
	/// Загрузить все объекты комнаты
	///
	AttachToRoom,
	DetachFromRoom,
}

impl C2SCommand {
	pub fn get_field_id(&self) -> Option<FieldId> {
		match self {
			C2SCommand::Create(_) => None,
			C2SCommand::Created(_) => None,
			C2SCommand::SetLong(command) => Some(command.field_id),
			C2SCommand::IncrementLongValue(command) => Some(command.field_id),
			C2SCommand::CompareAndSetLongValue(command) => Some(command.field_id),
			C2SCommand::SetFloat(command) => Some(command.field_id),
			C2SCommand::IncrementFloatCounter(command) => Some(command.field_id),
			C2SCommand::SetStruct(command) => Some(command.field_id),
			C2SCommand::Event(command) => Some(command.field_id),
			C2SCommand::TargetEvent(command) => Some(command.event.field_id),
			C2SCommand::Delete(_) => None,
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
		}
	}
	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			C2SCommand::Create(command) => Some(command.object_id.clone()),
			C2SCommand::Created(command) => Some(command.object_id.clone()),
			C2SCommand::SetLong(command) => Some(command.object_id.clone()),
			C2SCommand::IncrementLongValue(command) => Some(command.object_id.clone()),
			C2SCommand::CompareAndSetLongValue(command) => Some(command.object_id.clone()),
			C2SCommand::SetFloat(command) => Some(command.object_id.clone()),
			C2SCommand::IncrementFloatCounter(command) => Some(command.object_id.clone()),
			C2SCommand::SetStruct(command) => Some(command.object_id.clone()),
			C2SCommand::Event(command) => Some(command.object_id.clone()),
			C2SCommand::TargetEvent(command) => Some(command.event.object_id.clone()),
			C2SCommand::Delete(command) => Some(command.object_id.clone()),
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
		}
	}

	pub fn get_field_type(&self) -> Option<FieldType> {
		match self {
			C2SCommand::Create(_) => None,
			C2SCommand::Created(_) => None,
			C2SCommand::SetLong(_) => Some(FieldType::Long),
			C2SCommand::IncrementLongValue(_) => Some(FieldType::Long),
			C2SCommand::CompareAndSetLongValue(_) => Some(FieldType::Long),
			C2SCommand::SetFloat(_) => Some(FieldType::Double),
			C2SCommand::IncrementFloatCounter(_) => Some(FieldType::Double),
			C2SCommand::SetStruct(_) => Some(FieldType::Structure),
			C2SCommand::Event(_) => Some(FieldType::Event),
			C2SCommand::TargetEvent(_) => Some(FieldType::Event),
			C2SCommand::Delete(_) => None,
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
		}
	}

	pub fn get_type_id(&self) -> CommandTypeId {
		match self {
			C2SCommand::Create(_) => CommandTypeId(0),
			C2SCommand::Created(_) => CommandTypeId(1),
			C2SCommand::SetLong(_) => CommandTypeId(2),
			C2SCommand::IncrementLongValue(_) => CommandTypeId(3),
			C2SCommand::CompareAndSetLongValue(_) => CommandTypeId(4),
			C2SCommand::SetFloat(_) => CommandTypeId(5),
			C2SCommand::IncrementFloatCounter(_) => CommandTypeId(6),
			C2SCommand::SetStruct(_) => CommandTypeId(7),
			C2SCommand::Event(_) => CommandTypeId(8),
			C2SCommand::TargetEvent(_) => CommandTypeId(9),
			C2SCommand::Delete(_) => CommandTypeId(10),
			C2SCommand::AttachToRoom => CommandTypeId(11),
			C2SCommand::DetachFromRoom => CommandTypeId(12),
		}
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			C2SCommand::Create(command) => command.encode(out),
			C2SCommand::Created(_) => Ok(()),
			C2SCommand::SetLong(command) => command.encode(out),
			C2SCommand::IncrementLongValue(command) => command.encode(out),
			C2SCommand::CompareAndSetLongValue(command) => command.encode(out),
			C2SCommand::SetFloat(command) => command.encode(out),
			C2SCommand::IncrementFloatCounter(command) => command.encode(out),
			C2SCommand::SetStruct(command) => command.encode(out),
			C2SCommand::Event(command) => command.encode(out),
			C2SCommand::TargetEvent(command) => command.encode(out),
			C2SCommand::Delete(_) => Ok(()),
			C2SCommand::AttachToRoom => Ok(()),
			C2SCommand::DetachFromRoom => Ok(()),
		}
	}

	pub fn decode(
		command_type_id: CommandTypeId,
		context: &CommandContext,
		input: &mut Cursor<&mut [u8]>,
	) -> Result<C2SCommand, C2SCommandDecodeError> {
		match command_type_id {
			CommandTypeId(11) => return Ok(C2SCommand::AttachToRoom),
			CommandTypeId(12) => return Ok(C2SCommand::DetachFromRoom),
			_ => {}
		};

		let object_id = context.get_object_id()?.clone();
		match command_type_id {
			CommandTypeId(1) => return Ok(C2SCommand::Created(CreatedGameObjectCommand { object_id })),
			CommandTypeId(10) => return Ok(C2SCommand::Delete(DeleteGameObjectCommand { object_id })),
			_ => {}
		};

		let field_id = context.get_field_id()?;
		Ok(match command_type_id {
			CommandTypeId(0) => C2SCommand::Create(CreateGameObjectCommand::decode(object_id, input)?),
			CommandTypeId(2) => C2SCommand::SetLong(SetLongCommand::decode(object_id, field_id, input)?),
			CommandTypeId(3) => C2SCommand::IncrementLongValue(IncrementLongC2SCommand::decode(object_id, field_id, input)?),
			CommandTypeId(4) => C2SCommand::CompareAndSetLongValue(CompareAndSetLongCommand::decode(object_id, field_id, input)?),
			CommandTypeId(5) => C2SCommand::SetFloat(SetFloat64Command::decode(object_id, field_id, input)?),
			CommandTypeId(6) => {
				C2SCommand::IncrementFloatCounter(IncrementFloat64C2SCommand::decode(object_id, field_id, input)?)
			}
			CommandTypeId(7) => C2SCommand::SetStruct(StructureCommand::decode(object_id, field_id, input)?),
			CommandTypeId(8) => C2SCommand::Event(EventCommand::decode(object_id, field_id, input)?),
			CommandTypeId(9) => C2SCommand::TargetEvent(TargetEventCommand::decode(object_id, field_id, input)?),
			_ => Err(C2SCommandDecodeError::UnknownTypeId(command_type_id))?,
		})
	}
}

#[derive(Error, Debug)]
pub enum C2SCommandDecodeError {
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
