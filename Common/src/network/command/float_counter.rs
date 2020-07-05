use crate::constants::FieldID;
use crate::network::command::{CommandCode, Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};
use crate::room::object::ClientGameObjectId;

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq)]
pub struct IncrementFloat64CounterC2SCommand {
	pub object_id: ClientGameObjectId,
	pub field_id: FieldID,
	pub increment: f64,
}

///
/// Установка счетчика
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq)]
pub struct SetFloat64CounterCommand {
	pub object_id: ClientGameObjectId,
	pub field_id: FieldID,
	pub value: f64,
}


impl CommandCode for IncrementFloat64CounterC2SCommand {
	const COMMAND_CODE: u8 = 2;
}

impl CommandCode for SetFloat64CounterCommand {
	const COMMAND_CODE: u8 = 3;
}

impl Encoder for IncrementFloat64CounterC2SCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		self.object_id.encode(buffer)?;
		buffer.write_u16(self.field_id)?;
		buffer.write_f64(self.increment)?;
		Result::Ok(())
	}
}

impl Decoder for IncrementFloat64CounterC2SCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(IncrementFloat64CounterC2SCommand {
			object_id: ClientGameObjectId::decode(buffer)?,
			field_id: buffer.read_u16()?,
			increment: buffer.read_f64()?,
		})
	}
}

impl Encoder for SetFloat64CounterCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		self.object_id.encode(buffer)?;
		buffer.write_u16(self.field_id)?;
		buffer.write_f64(self.value)?;
		Result::Ok(())
	}
}

impl Decoder for SetFloat64CounterCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(SetFloat64CounterCommand {
			object_id: ClientGameObjectId::decode(buffer)?,
			field_id: buffer.read_u16()?,
			value: buffer.read_f64()?,
		})
	}
}