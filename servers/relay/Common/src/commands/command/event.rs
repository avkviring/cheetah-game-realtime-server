use serde::{Deserialize, Serialize};

use crate::commands::command::GameObjectCommand;
use crate::constants::FieldID;
use crate::room::object::GameObjectId;
use crate::room::fields::HeaplessBuffer;

///
/// Событие по объекту
/// - C->S, S->C
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldID,
	pub event: HeaplessBuffer,
}


impl GameObjectCommand for EventCommand {
	fn get_object_id(&self) -> &GameObjectId {
		&self.object_id
	}
}