use crate::constants::FieldID;
use crate::network::command::{CommandCode, Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};
use crate::room::object::GameObjectId;

///
/// Обновить структуру в обьекте
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq)]
pub struct StructureCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldID,
	pub structure: Vec<u8>,
}

impl CommandCode for StructureCommand {
	const COMMAND_CODE: u8 = 6;
}

impl Decoder for StructureCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(StructureCommand {
			object_id: GameObjectId::decode(buffer)?,
			field_id: buffer.read_u16()?,
			structure: buffer.read_to_vec_with_u16_size()?,
		})
	}
}

impl Encoder for StructureCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		self.object_id.encode(buffer)?;
		buffer.write_u16(self.field_id)?;
		buffer.write_u16(self.structure.len() as u16)?;
		buffer.write_bytes(&self.structure)?;
		Result::Ok(())
	}
}