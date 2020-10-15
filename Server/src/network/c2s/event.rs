use cheetah_relay_common::commands::command::event::EventCommand;

use crate::network::c2s::{get_field_and_change, ServerCommandExecutor, trace_c2s_command};
use crate::room::clients::Client;
use crate::room::Room;

impl ServerCommandExecutor for EventCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("Event", room, client, format!("params {:?}", self));
		get_field_and_change(
			"Event",
			room,
			client,
			&self.object_id,
			|room, object|
				{
					room.object_send_event(object, self.field_id, &self.event);
					format!("send event {} done", self.field_id)
				},
		);
	}
}