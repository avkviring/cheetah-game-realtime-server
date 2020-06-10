use crate::constants::{FieldID, GlobalObjectId};
use crate::network::command::{CommandCode, Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};

///
/// Событие по объекту
/// - C->S, S->C
#[derive(Debug, Clone, PartialEq)]
pub struct EventCommand {
	pub id: GlobalObjectId,
	pub field_id: FieldID,
	pub event: Vec<u8>,
}

impl CommandCode for EventCommand {
	const COMMAND_CODE: u8 = 1;
}

impl Decoder for EventCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(
			EventCommand {
				id: buffer.read_u64()?,
				field_id: buffer.read_u16()?,
				event: buffer.read_to_vec_with_u16_size()?,
			})
	}
}

impl Encoder for EventCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u64(self.id)?;
		buffer.write_u16(self.field_id)?;
		buffer.write_u16(self.event.len() as u16)?;
		buffer.write_bytes(&self.event)?;
		Result::Ok(())
	}
}