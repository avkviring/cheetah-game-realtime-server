use serde::{Deserialize, Serialize};

use crate::constants::FieldID;
use crate::room::object::ClientGameObjectId;

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct IncrementFloat64CounterC2SCommand {
	pub object_id: ClientGameObjectId,
	pub field_id: FieldID,
	pub increment: f64,
}

///
/// Установка счетчика
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetFloat64CounterCommand {
	pub object_id: ClientGameObjectId,
	pub field_id: FieldID,
	pub value: f64,
}