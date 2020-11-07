use cheetah_relay_common::commands::command::long_counter::{IncrementLongC2SCommand, SetLongCommand};
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::access::AccessGroups;


use crate::room::command::{CommandContext, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::object::server_object_id::ServerGameObjectId;
use crate::room::Room;

impl GameObject {
	pub fn set_long(&mut self, field_id: FieldID, value: i64) {
		self.fields.longs.insert(field_id, value);
	}
	
	
	pub fn get_long(&self, field_id: FieldID) -> i64 {
		*self.fields.longs.get(&field_id).unwrap_or(&0)
	}
}


impl ServerCommandExecutor for IncrementLongC2SCommand {
	fn execute(self, room: &mut Room, context: &CommandContext) {
		if let Some(object) = room.get_object(context.current_client.unwrap().public_key, &self.object_id) {
			let value = object.get_long(self.field_id);
			let new_value = value + self.increment;
			object.set_long(self.field_id, new_value);
			let access_groups = object.access_groups;
			let game_object_id = object.id.clone();
			send_update(room, access_groups, game_object_id, self.field_id, context, new_value);
		}
	}
}


impl ServerCommandExecutor for SetLongCommand {
	fn execute(self, room: &mut Room, context: &CommandContext) {
		if let Some(object) = room.get_object(context.current_client.unwrap().public_key, &self.object_id) {
			object.set_long(self.field_id, self.value);
			let access_groups = object.access_groups;
			let game_object_id = object.id.clone();
			send_update(room, access_groups, game_object_id, self.field_id, context, self.value);
		}
	}
}

fn send_update(
	room: &mut Room,
	access_groups: AccessGroups,
	game_object_id: ServerGameObjectId,
	field_id: FieldID,
	context: &CommandContext,
	new_value:
	i64,
) {
	room.send_to_clients(
		access_groups,
		game_object_id,
		context,
		|_, object_id| {
			S2CCommandUnion::SetLong(SetLongCommand {
				object_id,
				field_id,
				value: new_value,
			})
		},
	);
}
