use cheetah_relay_common::network::command::long_counter::{IncrementLongCounterC2SCommand, SetLongCounterCommand};

use crate::network::c2s::{get_field_and_change, ServerCommandExecutor, trace_c2s_command};
use crate::room::clients::Client;
use crate::room::objects::object::ObjectFieldType;
use crate::room::Room;

impl ServerCommandExecutor for IncrementLongCounterC2SCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("IncrementLongCounter", room, client, format!("params {:?}", self));
		get_field_and_change(
			"IncrementLongCounter",
			room,
			client,
			&self.object_id,
			self.field_id,
			ObjectFieldType::LongCounter,
			|room, object| {
				let value = room.object_increment_long_counter(object, self.field_id, self.increment);
				format!("increment done, result {}", value)
			},
		)
	}
}


impl ServerCommandExecutor for SetLongCounterCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("SetLongCounter", room, client, format!("params {:?}", self));
		get_field_and_change(
			"SetLongCounter",
			room,
			client,
			&self.object_id,
			self.field_id,
			ObjectFieldType::LongCounter,
			|room, object| {
				room.object_set_long_counter(object, self.field_id, self.value);
				format!("set done, result {}", self.value)
			},
		)
	}
}