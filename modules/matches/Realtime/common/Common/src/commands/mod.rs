use num_derive::FromPrimitive;
use thiserror::Error;

use crate::protocol::codec::commands::context::CommandContextError;

pub mod binary_value;
pub mod c2s;
pub mod field;
mod field_value;
pub mod s2c;
pub mod types;

pub use crate::commands::field::FieldType;
pub use crate::commands::field_value::FieldValue;

///
/// Идентификатор типа команды
///
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, FromPrimitive, Hash)]
pub enum CommandTypeId {
	CreateGameObject = 0,
	CreatedGameObject,
	SetLong,
	IncrementLong,
	CompareAndSetLong,
	SetDouble,
	IncrementDouble,
	SetStructure,
	Event,
	TargetEvent,
	Delete,
	AttachToRoom,
	DetachFromRoom,
	DeleteField,
	CompareAndSetStructure,
	Forwarded,
}

#[derive(Error, Debug)]
pub enum CommandDecodeError {
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
