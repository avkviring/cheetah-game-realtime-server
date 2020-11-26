use serde::{Deserialize, Serialize};

use crate::room::object::GameObjectId;

///
/// удаление игрового объекта
/// - направления C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteGameObjectCommand {
	pub object_id: GameObjectId
}