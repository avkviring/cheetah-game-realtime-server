use serde::{Deserialize, Serialize};

use crate::commands::command::GameObjectCommand;
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

impl GameObjectCommand for IncrementFloat64C2SCommand {
	fn get_object_id(&self) -> &GameObjectId {
		&self.object_id
	}
}

impl GameObjectCommand for SetFloat64Command {
	fn get_object_id(&self) -> &GameObjectId {
		&self.object_id
	}
}
