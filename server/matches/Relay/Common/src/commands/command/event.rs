use serde::{Deserialize, Serialize};

use crate::commands::command::HeaplessBuffer;
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

///
/// Событие по объекту для определенного пользователя
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetEventCommand {
	pub target: UserId,
	pub event: EventCommand,
}
