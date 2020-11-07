use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;

use crate::room::command::{CommandContext, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::object::server_object_id::ServerGameObjectId;
use crate::room::Room;

impl Room {
	pub fn delete_game_object(&mut self, id: &ServerGameObjectId, context: &CommandContext) {
		let object: Option<&mut GameObject> = self.objects.get_mut(id);
		match object {
			None => {
				log::error!("game object not found {:?}", id)
			}
			Some(object) => {
				let access_groups = object.access_groups;
				let game_object_id = object.id.clone();
				self.send_to_clients(
					access_groups,
					game_object_id,
					context,
					|_, object_id| {
						S2CCommandUnion::Delete(DeleteGameObjectCommand { object_id })
					},
				);
				self.objects.remove(id);
			}
		}
	}
}


impl ServerCommandExecutor for DeleteGameObjectCommand {
	fn execute(self, room: &mut Room, context: &CommandContext) {
		if let Some(object) = room.get_object(context.current_client.unwrap().public_key, &self.object_id) {
			let id = &object.id.clone();
			room.delete_game_object(id, context);
		}
	}
}

