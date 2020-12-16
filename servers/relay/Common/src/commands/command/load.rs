use serde::{Deserialize, Serialize};

use crate::constants::ObjectTemplateType;
use crate::room::access::AccessGroups;
use crate::room::object::GameObjectId;

///
/// Игровой объект создается
/// - направления C->S, S->C
///
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CreateGameObjectCommand {
	pub object_id: GameObjectId,
	pub template: ObjectTemplateType,
	pub access_groups: AccessGroups,
}

///
/// Игровой объект создан
/// - направления C->S, S->C
///
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CreatedGameObjectCommand {
	pub object_id: GameObjectId,
}
