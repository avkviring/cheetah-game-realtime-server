use crate::constants::FieldId;
use crate::room::object::GameObjectId;
use serde::{Deserialize, Serialize};

///
/// Обновить структуру в обьекте
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub structure: heapless::Vec<u8, 256>,
}
