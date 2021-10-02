use serde::{Deserialize, Serialize};

use crate::constants::FieldId;
use crate::room::object::GameObjectId;

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct IncrementLongC2SCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub increment: i64,
}

///
/// Установка счетчика
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetLongCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub value: i64,
}

///
/// Установка значения new если текущее равно current
/// reset - значение после выхода пользователя
///  
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompareAndSetLongCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub current: i64,
	pub new: i64,
	pub reset: i64,
}
