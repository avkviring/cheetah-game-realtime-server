use serde::{Deserialize, Serialize};

use crate::commands::command::{GameObjectCommand, HeaplessBuffer};
use crate::constants::FieldId;
use crate::room::object::GameObjectId;
use crate::room::UserId;

///
/// Событие по объекту
/// - C->S, S->C
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub event: HeaplessBuffer,
}

impl GameObjectCommand for EventCommand {
	fn get_object_id(&self) -> &GameObjectId {
		&self.object_id
	}
}

///
/// Событие по объекту для определенного пользователя
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetEventCommand {
	pub target: UserId,
	pub event: EventCommand,
}

impl GameObjectCommand for TargetEventCommand {
	fn get_object_id(&self) -> &GameObjectId {
		&self.event.object_id
	}
}
