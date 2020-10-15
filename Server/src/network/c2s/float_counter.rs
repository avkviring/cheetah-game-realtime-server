use cheetah_relay_common::commands::command::float_counter::{IncrementFloat64CounterC2SCommand, SetFloat64CounterCommand};

use crate::network::c2s::{get_field_and_change, ServerCommandExecutor, trace_c2s_command};
use crate::room::clients::Client;
use crate::room::Room;

impl ServerCommandExecutor for IncrementFloat64CounterC2SCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("IncrementFloatCounter", room, client, format!("params {:?}", self));
		get_field_and_change(
			"IncrementFloatCounter",
			room,
			client,
			&self.object_id,
			|room, object|
				{
					let value = room.object_increment_float_counter(object, self.field_id, self.increment);
					format!("increment done, result {}", value)
				},
		);
	}
}

impl ServerCommandExecutor for SetFloat64CounterCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("SetFloatCounter", room, client, format!("params {:?}", self));
		get_field_and_change(
			"SetFloatCounter",
			room,
			client,
			&self.object_id,
			|room, object|
				{
					room.object_set_float_counter(object, self.field_id, self.value);
					format!("set done, result {}", self.value)
				},
		);
	}
}
