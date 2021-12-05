use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

use crate::commands::command::event::{EventCommand, TargetEventCommand};
use crate::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
use crate::commands::command::load::{CreateGameObjectCommand, CreatedGameObjectCommand};
use crate::commands::command::long::{CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand};
use crate::commands::command::structure::StructureCommand;
use crate::commands::command::unload::DeleteGameObjectCommand;
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
