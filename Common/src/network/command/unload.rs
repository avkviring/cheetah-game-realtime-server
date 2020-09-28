use serde::{Deserialize, Serialize};

use crate::network::command::CommandCode;
use crate::room::object::ClientGameObjectId;

///
/// удаление игрового объекта
/// - направления C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnloadGameObjectCommand {
	pub object_id: ClientGameObjectId
}

impl CommandCode for UnloadGameObjectCommand {
	const COMMAND_CODE: u8 = 7;
}