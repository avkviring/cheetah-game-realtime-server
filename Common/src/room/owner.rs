use crate::constants::ClientId;
use crate::network::command::{Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};

///
/// владелец - клиент или root
///
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum ClientOwner {
	Root,
	CurrentClient,
	Client(ClientId),
}

impl Decoder for ClientOwner {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		let id = buffer.read_u16()?;
		Result::Ok(match id {
			0 => ClientOwner::Root,
			1 => ClientOwner::CurrentClient,
			client => ClientOwner::Client(client)
		})
	}
}

impl Encoder for ClientOwner {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u16(match self {
			ClientOwner::Root => { 0 }
			ClientOwner::CurrentClient => { 1 }
			ClientOwner::Client(code) => {
				debug_assert!(*code > 1);
				*code
			}
		})
	}
}



