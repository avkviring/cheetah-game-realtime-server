use std::io::{Cursor, Write};

use serde::{Deserialize, Serialize};

use crate::commands::HeaplessBuffer;
use crate::constants::FieldId;
use crate::protocol::codec::cursor::VariableInt;
use crate::room::object::GameObjectId;
use crate::room::RoomMemberId;

///
/// Событие по объекту
/// - C->S, S->C
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub event: HeaplessBuffer,
}

///
/// Событие по объекту для определенного пользователя
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetEventCommand {
	pub target: RoomMemberId,
	pub event: EventCommand,
}

impl EventCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.event.len() as u64)?;
		out.write_all(self.event.as_slice())
	}
}

impl TargetEventCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.target as u64)?;
		self.event.encode(out)
	}
}
