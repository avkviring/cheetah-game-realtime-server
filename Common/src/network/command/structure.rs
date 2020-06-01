use crate::constants::{FieldID, GlobalObjectId};
use crate::network::command::{CommandCode, Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};

///
/// Обновить структуру в обьекте
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq)]
pub struct SetStructCommand {
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub data: Vec<u8>,
}

impl CommandCode for SetStructCommand {
	const COMMAND_CODE: u8 = 6;
}

impl Decoder for SetStructCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(SetStructCommand {
			global_object_id: buffer.read_u64()?,
			field_id: buffer.read_u16()?,
			data: buffer.read_to_vec_with_u16_size()?,
		})
	}
}

impl Encoder for SetStructCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u64(self.global_object_id)?;
		buffer.write_u16(self.field_id)?;
		buffer.write_u16(self.data.len() as u16)?;
		buffer.write_bytes(&self.data)?;
		Result::Ok(())
	}
}