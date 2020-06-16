use crate::network::command::{CommandCode, Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};
use crate::room::object::ClientGameObjectId;

///
/// удаление игрового объекта
/// - направления C->S, S->C
///
#[derive(Debug, Clone, PartialEq)]
pub struct UnloadGameObjectCommand {
	pub object_id: ClientGameObjectId
}

impl CommandCode for UnloadGameObjectCommand {
	const COMMAND_CODE: u8 = 7;
}

impl Decoder for UnloadGameObjectCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(
			UnloadGameObjectCommand { object_id: ClientGameObjectId::decode(buffer)? }
		)
	}
}

impl Encoder for UnloadGameObjectCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		self.object_id.encode(buffer)
	}
}



