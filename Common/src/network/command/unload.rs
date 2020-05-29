use crate::network::command::{CommandCode, Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};

///
/// удаление игрового объекта
/// - направления C->S, S->C
///
#[derive(Debug, Clone, PartialEq)]
pub struct UnloadGameObjectCommand {
	pub global_object_id: u64
}

impl CommandCode for UnloadGameObjectCommand {
	const COMMAND_CODE: u8 = 7;
}

impl Decoder for UnloadGameObjectCommand {
	fn decode(bytes: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(
			UnloadGameObjectCommand { global_object_id: bytes.read_u64()? }
		)
	}
}

impl Encoder for UnloadGameObjectCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u64(self.global_object_id)
	}
}



