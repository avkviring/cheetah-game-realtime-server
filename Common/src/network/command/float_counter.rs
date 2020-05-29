use crate::constants::{FieldID, GlobalObjectId};
use crate::network::command::{CommandCode, Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug)]
pub struct IncrementFloatCounterC2SCommand {
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub increment: f64,
}

///
/// Установка счетчика
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq)]
pub struct SetFloatCounterCommand {
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub value: f64,
}


impl CommandCode for IncrementFloatCounterC2SCommand {
	const COMMAND_CODE: u8 = 2;
}

impl CommandCode for SetFloatCounterCommand {
	const COMMAND_CODE: u8 = 3;
}

impl Encoder for IncrementFloatCounterC2SCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u64(self.global_object_id)?;
		buffer.write_u16(self.field_id)?;
		buffer.write_f64(self.increment)?;
		Result::Ok(())
	}
}

impl Decoder for IncrementFloatCounterC2SCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(IncrementFloatCounterC2SCommand {
			global_object_id: buffer.read_u64()?,
			field_id: buffer.read_u16()?,
			increment: buffer.read_f64()?,
		})
	}
}

impl Encoder for SetFloatCounterCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u64(self.global_object_id)?;
		buffer.write_u16(self.field_id)?;
		buffer.write_f64(self.value)?;
		Result::Ok(())
	}
}

impl Decoder for SetFloatCounterCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(SetFloatCounterCommand {
			global_object_id: buffer.read_u64()?,
			field_id: buffer.read_u16()?,
			value: buffer.read_f64()?,
		})
	}
}