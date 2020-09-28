use crate::constants::FieldID;
use crate::network::command::CommandCode;
use crate::room::object::ClientGameObjectId;
use serde::{Deserialize, Serialize};

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct IncrementFloat64CounterC2SCommand {
	pub object_id: ClientGameObjectId,
	pub field_id: FieldID,
	pub increment: f64,
}

///
/// Установка счетчика
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq,Serialize,Deserialize)]
pub struct SetFloat64CounterCommand {
	pub object_id: ClientGameObjectId,
	pub field_id: FieldID,
	pub value: f64,
}


impl CommandCode for IncrementFloat64CounterC2SCommand {
	const COMMAND_CODE: u8 = 2;
}

impl CommandCode for SetFloat64CounterCommand {
	const COMMAND_CODE: u8 = 3;
}