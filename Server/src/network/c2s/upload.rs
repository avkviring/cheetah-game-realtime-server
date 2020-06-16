use cheetah_relay_common::network::command::upload::UploadGameObjectCommand;

use crate::network::c2s::{error_c2s_command, ServerCommandExecutor, trace_c2s_command};
use crate::room::clients::Client;
use crate::room::objects::id::ServerGameObjectId;
use crate::room::Room;

impl ServerCommandExecutor for UploadGameObjectCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("UploadGameObject", room, client, format!("{:?}", self));
		if self.access_groups.is_sub_groups(&client.configuration.groups) {
			let object_id = ServerGameObjectId::from_client_object_id(Option::Some(client.configuration.id), &self.object_id);
			match room.new_game_object(object_id, self.access_groups.clone(), self.fields) {
				Ok(_) => {
					trace_c2s_command(
						"UploadGameObject",
						room,
						client,
						format!("Object created with id {:?}", self.object_id),
					);
				}
				Err(_) => {
					error_c2s_command(
						"UploadGameObject",
						room,
						client,
						format!("Object already exists with id {:?}", self.object_id),
					);
				}
			}
		} else {
			error_c2s_command(
				"UploadGameObject",
				room,
				client,
				format!("Incorrect access group {:?} with client groups {:?}", self.access_groups, client.configuration.groups),
			);
		};
	}
}
