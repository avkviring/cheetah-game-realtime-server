use serde::{Deserialize, Serialize};

use crate::commands::command::event::{EventCommand, TargetEventCommand};
use crate::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
use crate::commands::command::load::{CreateGameObjectCommand, CreatedGameObjectCommand};
use crate::commands::command::long::{CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand};
use crate::commands::command::meta::c2s::C2SMetaCommandInformation;
use crate::commands::command::meta::s2c::S2CMetaCommandInformation;
use crate::commands::command::structure::StructureCommand;
use crate::commands::command::unload::DeleteGameObjectCommand;
use crate::room::object::GameObjectId;

pub mod event;
pub mod float;
pub mod load;
pub mod long;
pub mod meta;
pub mod structure;
pub mod unload;

pub type HeaplessBuffer = heapless::Vec<u8, heapless::consts::U256>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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
pub struct S2CCommandWithMeta {
	pub meta: S2CMetaCommandInformation,
	pub command: S2CCommand,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct C2SCommandWithMeta {
	pub meta: C2SMetaCommandInformation,
	pub command: C2SCommand,
}

///
/// Метод получения идентификатора объекта, для команд выполняемых от имени объекта
///
pub trait GameObjectCommand {
	fn get_object_id(&self) -> &GameObjectId;
}
