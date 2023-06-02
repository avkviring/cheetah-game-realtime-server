use num_derive::{FromPrimitive, ToPrimitive};
use thiserror::Error;

use crate::commands::c2s::C2SCommand;
use crate::commands::context::CommandContextError;
use crate::commands::guarantees::{ChannelSequence, ReliabilityGuarantees, ReliabilityGuaranteesChannel};
use crate::commands::s2c::S2CCommandWithCreator;
use crate::room::object::GameObjectId;

pub mod c2s;
pub mod codec;
pub mod context;
pub mod guarantees;
pub mod s2c;
pub mod types;

#[derive(Debug, PartialEq, Clone)]
pub struct CommandWithReliabilityGuarantees {
	pub reliability_guarantees: ReliabilityGuaranteesChannel,
	pub command: BothDirectionCommand,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CommandWithChannelType {
	pub command: BothDirectionCommand,
	pub channel_type: ReliabilityGuarantees,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum BothDirectionCommand {
	S2CWithCreator(S2CCommandWithCreator),
	C2S(C2SCommand),
}

impl ChannelSequence {
	pub const FIRST: ChannelSequence = ChannelSequence(0);

	#[must_use]
	pub fn next(&self) -> Self {
		ChannelSequence(self.0 + 1)
	}
}

impl BothDirectionCommand {
	#[must_use]
	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match &self {
			BothDirectionCommand::S2CWithCreator(command_with_meta) => command_with_meta.command.get_object_id(),
			BothDirectionCommand::C2S(command) => command.get_object_id(),
		}
	}
}

///
/// Идентификатор типа команды
///
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, FromPrimitive, ToPrimitive, Hash)]
pub enum CommandTypeId {
	CreateGameObject = 0,
	CreatedGameObject,
	SetLong,
	IncrementLong,
	SetDouble,
	IncrementDouble,
	SetStructure,
	SendEvent,
	TargetEvent,
	DeleteObject,
	AttachToRoom,
	DetachFromRoom,
	DeleteField,
	Forwarded,
	MemberConnected,
	MemberDisconnected,
}

#[derive(Error, Debug)]
pub enum CommandDecodeError {
	#[error("Unknown type {0:?}.")]
	UnknownTypeId(CommandTypeId),
	#[error("IO error {0}")]
	Io(#[from] std::io::Error),
	#[error("CommandContext error {0}")]
	CommandContext(#[from] CommandContextError),
}
