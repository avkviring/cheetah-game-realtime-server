use serde::{Deserialize, Serialize};

use crate::constants::FieldID;
use crate::network::command::CommandCode;
use crate::room::object::ClientGameObjectId;

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct IncrementLongCounterC2SCommand {
	pub object_id: ClientGameObjectId,
	pub field_id: FieldID,
	pub increment: i64,
}

///
/// Установка счетчика
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetLongCounterCommand {
	pub object_id: ClientGameObjectId,
	pub field_id: FieldID,
	pub value: i64,
}


impl CommandCode for IncrementLongCounterC2SCommand {
	const COMMAND_CODE: u8 = 4;
}

impl CommandCode for SetLongCounterCommand {
	const COMMAND_CODE: u8 = 5;
}