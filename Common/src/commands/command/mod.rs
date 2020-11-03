use serde::{Deserialize, Serialize};

use crate::commands::command::event::EventCommand;
use crate::commands::command::float_counter::{IncrementFloat64CounterC2SCommand, SetFloat64CounterCommand};
use crate::commands::command::load::LoadGameObjectCommand;
use crate::commands::command::long_counter::{IncrementLongCounterC2SCommand, SetLongCounterCommand};
use crate::commands::command::meta::c2s::C2SMetaCommandInformation;
use crate::commands::command::meta::s2c::S2CMetaCommandInformation;
use crate::commands::command::structure::StructureCommand;
use crate::commands::command::unload::UnloadGameObjectCommand;

pub mod event;
pub mod unload;
pub mod float_counter;
pub mod long_counter;
pub mod structure;
pub mod load;
pub mod meta;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum C2SCommandUnion {
	Load(LoadGameObjectCommand),
	SetLongCounter(SetLongCounterCommand),
	IncrementLongCounter(IncrementLongCounterC2SCommand),
	SetFloatCounter(SetFloat64CounterCommand),
	IncrementFloatCounter(IncrementFloat64CounterC2SCommand),
	Structure(StructureCommand),
	Event(EventCommand),
	Unload(UnloadGameObjectCommand),
	Test(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum S2CCommandUnion {
	Load(LoadGameObjectCommand),
	SetLongCounter(SetLongCounterCommand),
	SetFloatCounter(SetFloat64CounterCommand),
	SetStruct(StructureCommand),
	Event(EventCommand),
	Unload(UnloadGameObjectCommand),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct S2CCommandWithMeta {
	pub meta: S2CMetaCommandInformation,
	pub command: S2CCommandUnion,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct C2SCommandWithMeta {
	pub meta: C2SMetaCommandInformation,
	pub command: C2SCommandUnion,
}