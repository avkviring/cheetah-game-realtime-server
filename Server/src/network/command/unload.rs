use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::room::access::Access;

use crate::network::c2s::{error_c2s_command, ServerCommandExecutor, trace_c2s_command};
use crate::room::clients::Client;
use crate::room::objects::ErrorGetObjectWithCheckAccess;
use crate::room::room::Room;

impl ServerCommandExecutor for UnloadGameObjectCommand {
	fn execute(self, client: &Client, room: &mut Room) {
		trace_c2s_command("DeleteGameObject", room, client, format!("params {:?}", self));
		let result = room.get_object_with_check_access(Access::ROOT, client, self.global_object_id);
		match result {
			Ok(object) => {
				room.objects.delete_object(object.clone().borrow().id)
			}
			Err(error) => {
				match error {
					ErrorGetObjectWithCheckAccess::ObjectNotFound => {
						error_c2s_command("DeleteGameObject", room, client, format!("object not found {}", self.global_object_id));
					}
					ErrorGetObjectWithCheckAccess::AccessNotAllowed => {
						error_c2s_command("DeleteGameObject", room, client, format!("access not allowed {}", self.global_object_id));
					}
				}
			}
		}
	}
}
