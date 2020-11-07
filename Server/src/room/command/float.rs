use cheetah_relay_common::commands::command::float_counter::{IncrementFloat64C2SCommand, SetFloat64Command};
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::access::AccessGroups;

use crate::room::command::{CommandContext, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::object::server_object_id::ServerGameObjectId;
use crate::room::Room;

impl GameObject {
	pub fn set_float(&mut self, field_id: FieldID, value: f64) {
		self.fields.floats.insert(field_id, value);
	}
	
	pub fn get_float(&self, field_id: FieldID) -> f64 {
		*self.fields.floats.get(&field_id).unwrap_or(&0.0)
	}
}


impl ServerCommandExecutor for IncrementFloat64C2SCommand {
	fn execute(self, room: &mut Room, context: &CommandContext) {
		if let Some(object) = room.get_object(context.current_client.unwrap().public_key, &self.object_id) {
			let value = object.get_float(self.field_id);
			let new_value = value + self.increment;
			object.set_float(self.field_id, new_value);
			let access_groups = object.access_groups;
			let game_object_id = object.id.clone();
			send_update(room, access_groups, game_object_id, self.field_id, context, new_value);
		}
	}
}


impl ServerCommandExecutor for SetFloat64Command {
	fn execute(self, room: &mut Room, context: &CommandContext) {
		if let Some(object) = room.get_object(context.current_client.unwrap().public_key, &self.object_id) {
			object.set_float(self.field_id, self.value);
			let access_groups = object.access_groups;
			let game_object_id = object.id.clone();
			send_update(room, access_groups, game_object_id, self.field_id, context, self.value);
		}
	}
}

fn send_update(
	room: &mut Room,
	access_group: AccessGroups,
	game_object_id: ServerGameObjectId,
	field_id: FieldID,
	context: &CommandContext,
	new_value: f64,
) {
	room.send_to_clients(
		access_group,
		game_object_id,
		context,
		|_, object_id| {
			S2CCommandUnion::SetFloat64(SetFloat64Command {
				object_id,
				field_id,
				value: new_value,
			})
		},
	);
}
