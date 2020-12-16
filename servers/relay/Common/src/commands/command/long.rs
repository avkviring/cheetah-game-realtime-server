use serde::{Deserialize, Serialize};

use crate::commands::command::GameObjectCommand;
use crate::constants::FieldIDType;
use crate::room::object::GameObjectId;

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct IncrementLongC2SCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldIDType,
	pub increment: i64,
}

///
/// Установка счетчика
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetLongCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldIDType,
	pub value: i64,
}

///
/// Установка значения new если текущее равно current
/// reset - значение после выхода пользователя
///  
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompareAndSetLongCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldIDType,
	pub current: i64,
	pub new: i64,
	pub reset: i64,
}

impl GameObjectCommand for IncrementLongC2SCommand {
	fn get_object_id(&self) -> &GameObjectId {
		&self.object_id
	}
}

impl GameObjectCommand for SetLongCommand {
	fn get_object_id(&self) -> &GameObjectId {
		&self.object_id
	}
}

impl GameObjectCommand for CompareAndSetLongCommand {
	fn get_object_id(&self) -> &GameObjectId {
		&self.object_id
	}
}
