use cheetah_relay_common::network::command::structure::StructureCommand;

use crate::network::c2s::{get_field_and_change, ServerCommandExecutor, trace_c2s_command};
use crate::room::clients::Client;
use crate::room::objects::object::ObjectFieldType;
use crate::room::Room;

impl ServerCommandExecutor for StructureCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("UpdateStruct", room, client, format!("params {:?}", self));
		get_field_and_change(
			"UpdateStruct",
			room,
			client,
			self.global_object_id,
			self.field_id,
			ObjectFieldType::Struct,
			|room, object|
				{
					room.object_update_struct(object, self.field_id, self.structure);
					format!("update struct done")
				},
		);
	}
}
