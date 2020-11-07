use cheetah_relay_common::commands::command::event::EventCommand;
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::constants::FieldID;

use crate::room::command::{CommandContext, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

impl ServerCommandExecutor for EventCommand {
	fn execute(self, room: &mut Room, context: &CommandContext) {
		if let Some(object) = room.get_object(context.current_client.unwrap().public_key, &self.object_id) {
			let groups = object.access_groups;
			let object_id = object.id.clone();
			room.send_to_clients(
				groups,
				object_id,
				context,
				|_, object_id|
					S2CCommandUnion::Event(EventCommand {
						object_id,
						field_id: self.field_id,
						event: self.event.clone(),
					}),
			)
		}
	}
}