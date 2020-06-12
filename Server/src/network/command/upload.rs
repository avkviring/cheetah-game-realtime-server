use cheetah_relay_common::network::command::upload::UploadGameObjectC2SCommand;

use crate::network::c2s::{error_c2s_command, ServerCommandExecutor, trace_c2s_command};
use crate::room::clients::Client;
use crate::room::objects::CreateObjectError;
use crate::room::Room;

impl ServerCommandExecutor for UploadGameObjectC2SCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("UploadGameObject", room, client, format!("{:?}", self));
		let result = room.create_client_game_object(client, self.local_id, self.access_groups, self.fields);
		match result {
			Ok(id) => {
				trace_c2s_command("UploadGameObject", room, client, format!("Object created with id {}", id));
			}
			Err(error) => {
				match error {
					CreateObjectError::IncorrectGroups => {
						error_c2s_command("UploadGameObject", room, client, "Incorrect access group".to_string());
					}
				}
			}
		}
	}
}
