use serde::{Deserialize, Serialize};

use crate::commands::command::GameObjectCommand;
use crate::constants::FieldID;
use crate::room::object::GameObjectId;

///
/// Обновить структуру в обьекте
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureCommand {
    pub object_id: GameObjectId,
    pub field_id: FieldID,
    pub structure: Vec<u8>,
}

impl GameObjectCommand for StructureCommand {
    fn get_object_id(&self) -> &GameObjectId {
        &self.object_id
    }
}