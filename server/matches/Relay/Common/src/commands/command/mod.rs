use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

use crate::commands::command::event::{EventCommand, TargetEventCommand};
use crate::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
use crate::commands::command::load::{CreateGameObjectCommand, CreatedGameObjectCommand};
use crate::commands::command::long::{CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand};
use crate::commands::command::structure::StructureCommand;
use crate::commands::command::unload::DeleteGameObjectCommand;
use crate::constants::FieldId;
use crate::room::object::GameObjectId;
use crate::room::RoomMemberId;

pub mod event;
pub mod float;
pub mod load;
pub mod long;
pub mod structure;
pub mod unload;

pub type HeaplessBuffer = heapless::Vec<u8, 256>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AsRefStr)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AsRefStr)]
pub enum S2CCommand {
	Create(CreateGameObjectCommand),
	Created(CreatedGameObjectCommand),
	SetLong(SetLongCommand),
	SetFloat(SetFloat64Command),
	SetStruct(StructureCommand),
	Event(EventCommand),
	Delete(DeleteGameObjectCommand),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct S2CCommandWithCreator {
	pub creator: RoomMemberId,
	pub command: S2CCommand,
}

///
/// Тип данных поля
///
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum FieldType {
	Long,
	Double,
	Structure,
	Event,
}

impl C2SCommand {
	pub fn get_field_id(&self) -> Option<FieldId> {
		match self {
			C2SCommand::Create(_) => Option::None,
			C2SCommand::Created(_) => Option::None,
			C2SCommand::SetLong(command) => Some(command.field_id),
			C2SCommand::IncrementLongValue(command) => Some(command.field_id),
			C2SCommand::CompareAndSetLongValue(command) => Some(command.field_id),
			C2SCommand::SetFloat(command) => Some(command.field_id),
			C2SCommand::IncrementFloatCounter(command) => Some(command.field_id),
			C2SCommand::SetStruct(command) => Some(command.field_id),
			C2SCommand::Event(command) => Some(command.field_id),
			C2SCommand::TargetEvent(command) => Some(command.event.field_id),
			C2SCommand::Delete(_) => Option::None,
			C2SCommand::AttachToRoom => Option::None,
			C2SCommand::DetachFromRoom => Option::None,
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
			C2SCommand::AttachToRoom => Option::None,
			C2SCommand::DetachFromRoom => Option::None,
		}
	}

	pub fn get_field_type(&self) -> Option<FieldType> {
		match self {
			C2SCommand::Create(_) => Option::None,
			C2SCommand::Created(_) => Option::None,
			C2SCommand::SetLong(_) => Option::Some(FieldType::Long),
			C2SCommand::IncrementLongValue(_) => Option::Some(FieldType::Long),
			C2SCommand::CompareAndSetLongValue(_) => Option::Some(FieldType::Long),
			C2SCommand::SetFloat(_) => Option::Some(FieldType::Double),
			C2SCommand::IncrementFloatCounter(_) => Option::Some(FieldType::Double),
			C2SCommand::SetStruct(_) => Option::Some(FieldType::Structure),
			C2SCommand::Event(_) => Option::Some(FieldType::Event),
			C2SCommand::TargetEvent(_) => Option::Some(FieldType::Event),
			C2SCommand::Delete(_) => Option::None,
			C2SCommand::AttachToRoom => Option::None,
			C2SCommand::DetachFromRoom => Option::None,
		}
	}
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
}
