use serde::{Deserialize, Serialize};

use crate::room::owner::ClientOwner;

///
/// Идентификатор игрового объекта на клиенте
///
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct GameObjectId {
	///
	/// Создатель игрового объекта
	///
	pub owner: ClientOwner,
	
	///
	/// Идентификатор игрового объекта в рамках владельца
	///
	pub id: u32,
}

impl GameObjectId {
	pub fn new(id: u32, owner: ClientOwner) -> Self {
		GameObjectId {
			owner,
			id,
		}
	}
}

impl Default for GameObjectId {
	fn default() -> Self {
		GameObjectId::new(0, ClientOwner::Root)
	}
}