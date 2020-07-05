use crate::network::command::{Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};
use crate::room::owner::ClientOwner;

///
/// Идентификатор игрового объекта на клиенте
///
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ClientGameObjectId {
	///
	/// Создатель игрового объекта
	///
	pub owner: ClientOwner,
	
	///
	/// Идентификатор игрового объекта в рамках владельца
	///
	pub id: u32,
}

impl ClientGameObjectId {
	pub fn new(id: u32, owner: ClientOwner) -> Self {
		ClientGameObjectId {
			owner,
			id,
		}
	}
}

impl Decoder for ClientGameObjectId {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		let id = buffer.read_u32()?;
		let owner = ClientOwner::decode(buffer)?;
		Result::Ok(ClientGameObjectId::new(id, owner))
	}
}

impl Encoder for ClientGameObjectId {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u32(self.id)?;
		self.owner.encode(buffer)?;
		Result::Ok(())
	}
}

impl Default for ClientGameObjectId {
	fn default() -> Self {
		ClientGameObjectId::new(0, ClientOwner::Root)
	}
}