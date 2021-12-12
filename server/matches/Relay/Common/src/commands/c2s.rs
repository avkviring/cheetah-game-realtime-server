use std::io::Cursor;

use crate::commands::types::event::{EventCommand, TargetEventCommand};
use crate::commands::types::float::{IncrementFloat64C2SCommand, SetFloat64Command};
use crate::commands::types::load::{CreateGameObjectCommand, CreatedGameObjectCommand};
use crate::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand};
use crate::commands::types::structure::StructureCommand;
use crate::commands::types::unload::DeleteGameObjectCommand;
use crate::commands::FieldType;
use crate::constants::FieldId;
use crate::room::object::GameObjectId;
use strum_macros::AsRefStr;

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
	pub(crate) fn decode(p0: u8, p1: Option<GameObjectId>, p2: Option<FieldId>, p3: &mut Cursor<&mut [u8]>) -> C2SCommand {
		todo!()
	}
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

	pub fn get_type_id(&self) -> u8 {
		match self {
			C2SCommand::Create(_) => 0,
			C2SCommand::Created(_) => 1,
			C2SCommand::SetLong(_) => 2,
			C2SCommand::IncrementLongValue(_) => 3,
			C2SCommand::CompareAndSetLongValue(_) => 4,
			C2SCommand::SetFloat(_) => 5,
			C2SCommand::IncrementFloatCounter(_) => 6,
			C2SCommand::SetStruct(_) => 7,
			C2SCommand::Event(_) => 8,
			C2SCommand::TargetEvent(_) => 9,
			C2SCommand::Delete(_) => 10,
			C2SCommand::AttachToRoom => 11,
			C2SCommand::DetachFromRoom => 12,
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
}
