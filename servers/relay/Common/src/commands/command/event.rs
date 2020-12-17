use serde::{Deserialize, Serialize};

use crate::commands::command::{GameObjectCommand, HeaplessBuffer};
use crate::constants::FieldIdType;
use crate::room::object::GameObjectId;

///
/// Событие по объекту
/// - C->S, S->C
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldIdType,
	pub event: HeaplessBuffer,
}

impl GameObjectCommand for EventCommand {
	fn get_object_id(&self) -> &GameObjectId {
		&self.object_id
	}
}
