use serde::{Deserialize, Serialize};

use crate::constants::FieldId;
use crate::room::object::GameObjectId;

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct IncrementFloat64C2SCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub increment: f64,
}

///
/// Установка счетчика
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetFloat64Command {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub value: f64,
}
