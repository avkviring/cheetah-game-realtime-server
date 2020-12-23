use serde::{Deserialize, Serialize};

use crate::room::owner::ObjectOwner;

///
/// Идентификатор игрового объекта на клиенте
///
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct GameObjectId {
	///
	/// Создатель игрового объекта
	///
	pub owner: ObjectOwner,

	///
	/// Идентификатор игрового объекта в рамках владельца
	///
	pub id: u32,
}

impl GameObjectId {
	///
	/// Идентификатор первого клиентского объекта (для исключения пересечений с объектами клиента из конфигурации)
	///
	pub const CLIENT_OBJECT_ID_OFFSET: u32 = 512;

	pub fn new(id: u32, owner: ObjectOwner) -> Self {
		GameObjectId { owner, id }
	}
}

impl Default for GameObjectId {
	fn default() -> Self {
		GameObjectId::new(0, ObjectOwner::Root)
	}
}
