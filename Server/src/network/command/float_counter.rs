use cheetah_relay_common::network::command::float_counter::{IncrementFloatCounterC2SCommand, SetFloatCounterCommand};

use crate::network::c2s::{get_field_and_change, ServerCommandExecutor, trace_c2s_command};
use crate::room::clients::Client;
use crate::room::objects::object::ObjectFieldType;
use crate::room::room::Room;

impl ServerCommandExecutor for IncrementFloatCounterC2SCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("IncrementFloatCounter", room, client, format!("params {:?}", self));
		get_field_and_change(
			"IncrementFloatCounter",
			room,
			client,
			self.global_object_id,
			self.field_id,
			ObjectFieldType::FloatCounter,
			|room, object|
				{
					let value = room.object_increment_float_counter(object, self.field_id, self.increment);
					format!("increment done, result {}", value)
				},
		);
	}
}

impl ServerCommandExecutor for SetFloatCounterCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("SetFloatCounter", room, client, format!("params {:?}", self));
		get_field_and_change(
			"SetFloatCounter",
			room,
			client,
			self.global_object_id,
			self.field_id,
			ObjectFieldType::FloatCounter,
			|room, object|
				{
					room.object_set_float_counter(object, self.field_id, self.value);
					format!("set done, result {}", self.value)
				},
		);
	}
}
