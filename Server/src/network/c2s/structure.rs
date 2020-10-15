use cheetah_relay_common::commands::command::structure::StructureCommand;

use crate::network::c2s::{get_field_and_change, ServerCommandExecutor, trace_c2s_command};
use crate::room::clients::Client;
use crate::room::Room;

impl ServerCommandExecutor for StructureCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("UpdateStruct", room, client, format!("params {:?}", &self));
		
		let object_id = &self.object_id;
		let field_id = self.field_id;
		let structure = self.structure.clone();
		
		get_field_and_change(
			"UpdateStruct",
			room,
			client,
			object_id,
			|room, object|
				{
					room.object_update_struct(object, field_id, structure);
					"update struct done".to_string()
				},
		);
	}
}
