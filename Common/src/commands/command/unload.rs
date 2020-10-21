use serde::{Deserialize, Serialize};

use crate::room::object::ClientGameObjectId;

///
/// удаление игрового объекта
/// - направления C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnloadGameObjectCommand {
	pub object_id: ClientGameObjectId
}