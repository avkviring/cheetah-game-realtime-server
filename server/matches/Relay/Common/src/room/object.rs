use std::io::Cursor;

use crate::room::owner::GameObjectOwner;

///
/// Идентификатор игрового объекта на клиенте
///
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct GameObjectId {
	///
	/// Создатель игрового объекта
	///
	pub owner: GameObjectOwner,

	///
	/// Идентификатор игрового объекта в рамках владельца
	///
	pub id: u32,
}

impl GameObjectId {
	pub fn encode(&self, p0: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		todo!()
	}
	pub fn decode(input: &mut Cursor<&mut [u8]>) -> std::io::Result<Self> {
		todo!()
	}
}

impl GameObjectId {
	///
	/// Идентификатор первого клиентского объекта (для исключения пересечений с объектами клиента из конфигурации)
	///
	pub const CLIENT_OBJECT_ID_OFFSET: u32 = 512;

	pub fn new(id: u32, owner: GameObjectOwner) -> Self {
		GameObjectId { owner, id }
	}
}

impl Default for GameObjectId {
	fn default() -> Self {
		GameObjectId::new(0, GameObjectOwner::Room)
	}
}
