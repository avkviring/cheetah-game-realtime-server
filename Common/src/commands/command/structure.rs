use serde::{Deserialize, Serialize};

use crate::constants::FieldID;
use crate::room::object::ClientGameObjectId;

///
/// Обновить структуру в обьекте
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureCommand {
	pub object_id: ClientGameObjectId,
	pub field_id: FieldID,
	pub structure: Vec<u8>,
}