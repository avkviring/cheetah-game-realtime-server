use crate::constants::ClientId;
use crate::network::command::{Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};
use crate::room::owner::Owner;

///
/// Идентификатор игрового объекта
///
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct GameObjectId {
	///
	/// Создатель игрового объекта
	///
	pub owner: Owner,
	
	///
	/// Идентификатор игрового объекта в рамках владельца
	///
	pub id: u32,
}

impl GameObjectId {
	pub fn new(id: u32, owner: Owner) -> Self {
		GameObjectId {
			owner,
			id,
		}
	}
}

impl Decoder for GameObjectId {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		let id = buffer.read_u32()?;
		let owner = Owner::decode(buffer)?;
		Result::Ok(GameObjectId::new(id, owner))
	}
}

impl Encoder for GameObjectId {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u32(self.id)?;
		self.owner.encode(buffer)?;
		Result::Ok(())
	}
}

impl Default for GameObjectId {
	fn default() -> Self {
		GameObjectId::new(0, Owner::Root)
	}
}