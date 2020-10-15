use serde::{Deserialize, Serialize};

use crate::constants::FieldID;
use crate::room::object::ClientGameObjectId;

///
/// Событие по объекту
/// - C->S, S->C
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventCommand {
	pub object_id: ClientGameObjectId,
	pub field_id: FieldID,
	pub event: Vec<u8>,
}