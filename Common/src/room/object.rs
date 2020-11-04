use serde::{Deserialize, Serialize};
use crate::room::owner::ClientOwner;

///
/// Идентификатор игрового объекта на клиенте
///
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
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
impl Default for ClientGameObjectId {
	fn default() -> Self {
		ClientGameObjectId::new(0, ClientOwner::Root)
	}
}