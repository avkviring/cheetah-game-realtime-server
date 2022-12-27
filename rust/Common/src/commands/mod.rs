use num_derive::{FromPrimitive, ToPrimitive};
use thiserror::Error;

pub use crate::commands::field::FieldType;
use crate::protocol::codec::commands::context::CommandContextError;

pub mod binary_value;
pub mod c2s;
pub mod field;
pub mod s2c;
pub mod types;

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
