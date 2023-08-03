use crate::room::object::GameObjectId;
use serde::{Deserialize, Serialize};

///
/// удаление игрового объекта
/// - направления C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct DeleteGameObjectCommand {
	pub object_id: GameObjectId,
}
