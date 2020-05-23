use crate::network::command::c2s::error_c2s_command;
use crate::network::command::c2s::trace_c2s_command;
use crate::room::clients::Client;
use crate::room::groups::Access;
use crate::room::objects::ErrorGetObjectWithCheckAccess;
use crate::room::room::Room;
use crate::network::types::niobuffer::NioBuffer;

/// удаление игрового объекта
#[derive(Debug)]
pub struct DeleteGameObjectC2SCommand {
	pub global_object_id: u64
}


impl DeleteGameObjectC2SCommand {
	pub const COMMAND_ID: u8 = 2;
	
	pub fn decode(bytes: &mut NioBuffer) -> Option<DeleteGameObjectC2SCommand> {
		bytes
			.read_u64()
			.map(|id| DeleteGameObjectC2SCommand { global_object_id: id })
			.ok()
	}
	
	pub fn execute(&self, client: &Client, room: &mut Room) {
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


