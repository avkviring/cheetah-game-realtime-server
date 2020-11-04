use serde::{Deserialize, Serialize};

use crate::room::object::ClientGameObjectId;

///
/// удаление игрового объекта
/// - направления C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteGameObjectCommand {
	pub object_id: ClientGameObjectId
}