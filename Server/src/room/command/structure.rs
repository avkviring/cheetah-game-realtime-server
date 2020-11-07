use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::command::structure::StructureCommand;
use cheetah_relay_common::constants::FieldID;

use crate::room::command::{CommandContext, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

impl GameObject {
	pub fn set_structure(&mut self, field_id: FieldID, structure: Vec<u8>, context: &CommandContext) {
		self.fields.structures.insert(field_id, structure.clone());
	}
}


impl ServerCommandExecutor for StructureCommand {
	fn execute(self, room: &mut Room, context: &CommandContext) {
		if let Some(object) = room.get_object(context.current_client.unwrap().public_key, &self.object_id) {
			let field_id = self.field_id;
			let structure = self.structure.clone();
			object.set_structure(field_id, structure.clone(), context);
			let access_group = object.access_groups;
			let game_object_id = object.id.clone();
			room.send_to_clients(
				access_group,
				game_object_id,
				context,
				|_, object_id|
					S2CCommandUnion::SetStruct(StructureCommand {
						object_id,
						field_id,
						structure: structure.clone(),
					}),
			)
		}
	}
}
