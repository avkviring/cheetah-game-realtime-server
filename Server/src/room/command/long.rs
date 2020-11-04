use cheetah_relay_common::commands::command::long_counter::{IncrementLongC2SCommand, SetLongCommand};
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::object::ClientGameObjectId;

use crate::room::command::{CommandContext, ServerObjectCommandExecutor, ServerRoomCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

impl GameObject {
    pub fn set_long(&mut self, field_id: FieldID, value: i64, context: &CommandContext) {
        self.fields.longs.insert(field_id, value);
        self.send_to_clients(context, |_, object_id| {
            S2CCommandUnion::SetLong(SetLongCommand {
                object_id,
                field_id,
                value,
            })
        });
    }


    pub fn get_long(&self, field_id: FieldID) -> i64 {
        *self.fields.longs.get(&field_id).unwrap_or(&0)
    }
}


impl ServerObjectCommandExecutor for IncrementLongC2SCommand {
    fn execute(self, object: &mut GameObject, context: &CommandContext) {
        let value = object.get_long(self.field_id);
        object.set_long(self.field_id, value + self.increment, context);
    }
}


impl ServerObjectCommandExecutor for SetLongCommand {
    fn execute(self, object: &mut GameObject, context: &CommandContext) {
        object.set_long(self.field_id, self.value, context);
    }
}