use std::io::{Cursor, Error, ErrorKind, Read, Write};

use crate::commands::CommandBuffer;
use crate::constants::FieldId;
use crate::protocol::codec::cursor::VariableInt;
use crate::room::object::GameObjectId;

///
/// Обновить структуру в обьекте
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq)]
pub struct SetStructureCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub structure: CommandBuffer,
}
impl SetStructureCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.structure.len() as u64)?;
		out.write_all(self.structure.as_slice())
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&mut [u8]>) -> std::io::Result<Self> {
		let size = input.read_variable_u64()? as usize;
		let mut structure = CommandBuffer::new();
		if size > structure.capacity() {
			return Err(Error::new(
				ErrorKind::InvalidData,
				format!("Structure buffer size to big {}", size),
			));
		}
		unsafe {
			structure.set_len(size);
		}
		input.read_exact(&mut structure[0..size])?;

		Ok(Self {
			object_id,
			field_id,
			structure,
		})
	}
}
