use crate::room::access::AccessGroups;
use crate::room::fields::GameObjectFields;
use crate::room::object::GameObjectId;
use serde::{Deserialize, Serialize};


///
/// Создание игрового объекта
/// - направления C->S, S->C
///
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CreateGameObjectCommand {
	pub object_id: GameObjectId,
	pub template: u16,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFields,
}
