use cheetah_relay_common::commands::command::event::EventCommand;
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::constants::FieldID;

use crate::room::command::{CommandContext, ServerObjectCommandExecutor, ServerRoomCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

impl GameObject {
    pub fn send_event(&mut self, field_id: FieldID, event: Vec<u8>, context: &CommandContext) {
        self.send_to_clients(
            context,
            |_, object_id|
                S2CCommandUnion::Event(EventCommand {
                    object_id,
                    field_id,
                    event: event.clone(),
                }),
        )
    }
}

impl ServerObjectCommandExecutor for EventCommand {
    fn execute(self, object: &mut GameObject, context: &CommandContext) {
        object.send_event(self.field_id, self.event, context);
    }
}