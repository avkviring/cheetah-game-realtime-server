use crate::constants::FieldID;
use crate::network::command::{CommandCode, Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};
use crate::room::object::GameObjectId;

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq)]
pub struct IncrementLongCounterC2SCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldID,
	pub increment: i64,
}

///
/// Установка счетчика
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq)]
pub struct SetLongCounterCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldID,
	pub value: i64,
}


impl CommandCode for IncrementLongCounterC2SCommand {
	const COMMAND_CODE: u8 = 4;
}

impl CommandCode for SetLongCounterCommand {
	const COMMAND_CODE: u8 = 5;
}

impl Encoder for IncrementLongCounterC2SCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		self.object_id.encode(buffer)?;
		buffer.write_u16(self.field_id)?;
		buffer.write_i64(self.increment)?;
		Result::Ok(())
	}
}

impl Decoder for IncrementLongCounterC2SCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(IncrementLongCounterC2SCommand {
			object_id: GameObjectId::decode(buffer)?,
			field_id: buffer.read_u16()?,
			increment: buffer.read_i64()?,
		})
	}
}


impl Encoder for SetLongCounterCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		self.object_id.encode(buffer)?;
		buffer.write_u16(self.field_id)?;
		buffer.write_i64(self.value)?;
		Result::Ok(())
	}
}

impl Decoder for SetLongCounterCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(SetLongCounterCommand {
			object_id: GameObjectId::decode(buffer)?,
			field_id: buffer.read_u16()?,
			value: buffer.read_i64()?,
		})
	}
}