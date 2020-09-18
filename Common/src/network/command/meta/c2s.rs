use crate::network::command::{Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};

///
/// Служебная информация для каждой входящей команды
///
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct C2SMetaCommandInformation {
	pub command_code: u8,
	///
	/// Условное время создания команды на клиенте
	///
	pub timestamp: u64,
}


impl C2SMetaCommandInformation {
	pub fn new(command_code: u8, timestamp: u64) -> Self {
		Self {
			command_code,
			timestamp,
		}
	}
}

impl Decoder for C2SMetaCommandInformation {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(
			Self {
				command_code: buffer.read_u8()?,
				timestamp: buffer.read_u64()?,
			}
		)
	}
}

impl Encoder for C2SMetaCommandInformation {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u8(self.command_code)?;
		buffer.write_u64(self.timestamp)?;
		Result::Ok(())
	}
}