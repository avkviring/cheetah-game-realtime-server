use crate::constants::ClientId;
use crate::network::command::{Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};

///
/// владелец - клиент или root
///
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Owner {
	Root,
	CurrentClient,
	Client(ClientId),
}

impl Decoder for Owner {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		let id = buffer.read_u16()?;
		Result::Ok(match id {
			0 => Owner::Root,
			1 => Owner::CurrentClient,
			client => Owner::Client(client)
		})
	}
}

impl Encoder for Owner {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u16(match self {
			Owner::Root => { 0 }
			Owner::CurrentClient => { 1 }
			Owner::Client(code) => {
				debug_assert!(*code > 1);
				*code
			}
		})
	}
}



