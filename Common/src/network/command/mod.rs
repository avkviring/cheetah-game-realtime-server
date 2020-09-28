use crate::network::command::event::EventCommand;
use crate::network::command::float_counter::{IncrementFloat64CounterC2SCommand, SetFloat64CounterCommand};
use crate::network::command::load::LoadGameObjectCommand;
use crate::network::command::long_counter::{IncrementLongCounterC2SCommand, SetLongCounterCommand};
use crate::network::command::meta::c2s::C2SMetaCommandInformation;
use crate::network::command::meta::s2c::S2CMetaCommandInformation;
use crate::network::command::structure::StructureCommand;
use crate::network::command::unload::UnloadGameObjectCommand;


pub mod event;
pub mod unload;
pub mod float_counter;
pub mod long_counter;
pub mod structure;
pub mod load;
pub mod meta;

pub trait CommandCode {
	const COMMAND_CODE: u8;
}

#[derive(Debug)]
pub enum C2SCommandUnion {
	Load(LoadGameObjectCommand),
	SetLongCounter(SetLongCounterCommand),
	IncrementLongCounter(IncrementLongCounterC2SCommand),
	SetFloatCounter(SetFloat64CounterCommand),
	IncrementFloatCounter(IncrementFloat64CounterC2SCommand),
	Structure(StructureCommand),
	Event(EventCommand),
	Unload(UnloadGameObjectCommand),
}

#[derive(Debug, PartialEq)]
pub enum S2CCommandUnion {
	Load(LoadGameObjectCommand),
	SetLongCounter(SetLongCounterCommand),
	SetFloatCounter(SetFloat64CounterCommand),
	SetStruct(StructureCommand),
	Event(EventCommand),
	Unload(UnloadGameObjectCommand),
}

#[derive(Debug, PartialEq)]
pub struct S2CCommandWithMeta {
	pub meta: S2CMetaCommandInformation,
	pub command: S2CCommandUnion,
}

#[derive(Debug)]
pub struct C2SCommandWithMeta {
	pub meta: C2SMetaCommandInformation,
	pub command: C2SCommandUnion,
}


impl S2CCommandUnion {
	pub fn get_code(&self) -> u8 {
		match self {
			S2CCommandUnion::Load(_) => LoadGameObjectCommand::COMMAND_CODE,
			S2CCommandUnion::Unload(_) => UnloadGameObjectCommand::COMMAND_CODE,
			S2CCommandUnion::SetLongCounter(_) => SetLongCounterCommand::COMMAND_CODE,
			S2CCommandUnion::SetFloatCounter(_) => SetFloat64CounterCommand::COMMAND_CODE,
			S2CCommandUnion::Event(_) => EventCommand::COMMAND_CODE,
			S2CCommandUnion::SetStruct(_) => StructureCommand::COMMAND_CODE,
		}
	}
}
