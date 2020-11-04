use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::command::structure::StructureCommand;
use cheetah_relay_common::constants::FieldID;

use crate::room::command::{CommandContext, ServerObjectCommandExecutor, ServerRoomCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

impl GameObject {
    pub fn set_structure(&mut self, field_id: FieldID, structure: Vec<u8>, context: &CommandContext) {
        self.fields.structures.insert(field_id, structure.clone());
        self.send_to_clients(
            context,
            |_, object_id|
                S2CCommandUnion::SetStruct(StructureCommand {
                    object_id,
                    field_id,
                    structure: structure.clone(),
                }),
        )
    }
}


impl ServerObjectCommandExecutor for StructureCommand {
    fn execute(self, object: &mut GameObject, context: &CommandContext) {
        let field_id = self.field_id;
        let structure = self.structure.clone();
        object.set_structure(field_id, structure, context);
    }
}
