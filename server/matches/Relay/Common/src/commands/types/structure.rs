use std::io::{Cursor, Write};

use crate::commands::HeaplessBuffer;
use serde::{Deserialize, Serialize};

use crate::constants::FieldId;
use crate::protocol::codec::cursor::VariableInt;
use crate::room::object::GameObjectId;

///
/// Обновить структуру в обьекте
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub structure: HeaplessBuffer,
}
impl StructureCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.structure.len() as u64)?;
		out.write_all(self.structure.as_slice())
	}
}
