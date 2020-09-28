use crate::network::command::CommandCode;
use crate::room::access::AccessGroups;
use crate::room::fields::GameObjectFields;
use crate::room::object::ClientGameObjectId;
use serde::{Deserialize, Serialize};


///
/// Загрузка объекта
/// - направления C->S, S->C
///
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct LoadGameObjectCommand {
	pub object_id: ClientGameObjectId,
	pub template: u16,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFields,
}


impl CommandCode for LoadGameObjectCommand {
	const COMMAND_CODE: u8 = 8;
}