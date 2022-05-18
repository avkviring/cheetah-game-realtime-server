use std::io::Cursor;
use std::io::ErrorKind;

use byteorder::{ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::protocol::codec::commands::context::CommandContextError;

pub mod binary_value;
pub mod c2s;
pub mod s2c;
pub mod types;

///
/// Идентификатор типа команды
///
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CommandTypeId(pub u8);

impl CommandTypeId {
	const CREATE_GAME_OBJECT: CommandTypeId = CommandTypeId(0);
	const CREATED_GAME_OBJECT: CommandTypeId = CommandTypeId(1);
	const SET_LONG: CommandTypeId = CommandTypeId(2);
	const INCREMENT_LONG: CommandTypeId = CommandTypeId(3);
	const COMPARE_AND_SET_LONG: CommandTypeId = CommandTypeId(4);
	const SET_DOUBLE: CommandTypeId = CommandTypeId(5);
	const INCREMENT_DOUBLE: CommandTypeId = CommandTypeId(6);
	const SET_STRUCTURE: CommandTypeId = CommandTypeId(7);
	const EVENT: CommandTypeId = CommandTypeId(8);
	const TARGET_EVENT: CommandTypeId = CommandTypeId(9);
	const DELETE: CommandTypeId = CommandTypeId(10);
	const ATTACH_TO_ROOM: CommandTypeId = CommandTypeId(11);
	const DETACH_FROM_ROOM: CommandTypeId = CommandTypeId(12);
	const DELETE_FIELD: CommandTypeId = CommandTypeId(13);
	const COMPARE_AND_SET_STRUCTURE: CommandTypeId = CommandTypeId(14);
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

impl From<FieldType> for &str {
	fn from(source: FieldType) -> Self {
		match source {
			FieldType::Long => "long",
			FieldType::Double => "double",
			FieldType::Structure => "structure",
			FieldType::Event => "event",
		}
	}
}

impl FieldType {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		let code = match self {
			FieldType::Long => 1,
			FieldType::Double => 2,
			FieldType::Structure => 3,
			FieldType::Event => 4,
		};
		out.write_u8(code)
	}
	pub fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let value = input.read_u8()?;
		Ok(match value {
			1 => FieldType::Long,
			2 => FieldType::Double,
			3 => FieldType::Structure,
			4 => FieldType::Event,
			_ => return Err(std::io::Error::new(ErrorKind::InvalidData, format!("{}", value))),
		})
	}
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
