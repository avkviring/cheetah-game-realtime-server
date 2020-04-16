use bytebuffer::ByteBuffer;
use log::error;
use log::trace;

use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor};
use crate::relay::room::clients::Client;
use crate::relay::room::groups::Access;
use crate::relay::room::objects::ErrorGetObjectWithCheckAccess;
use crate::relay::room::room::Room;

/// удаление игрового объекта
#[derive(Debug)]
pub struct DeleteGameObjectC2SCommand {
	pub global_object_id: u64
}


impl C2SCommandDecoder for DeleteGameObjectC2SCommand {
	const COMMAND_ID: u8 = 2;
	fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn C2SCommandExecutor>> {
		return match bytes.read_u64() {
			Ok(id) => {
				Option::Some(Box::new(DeleteGameObjectC2SCommand { global_object_id: id }))
			}
			Err(_) => Option::None,
		};
	}
}

impl C2SCommandExecutor for DeleteGameObjectC2SCommand {
	fn execute(&self, client: &Client, room: &mut Room) {
		trace!("C2S:\tDeleteGameObject : client {} params {:?}", client.configuration.hash, self);
		let result = room.get_object_with_check_access(Access::ROOT, client, self.global_object_id);
		match result {
			Ok(object) => {
				room.objects.delete_object(object.id)
			}
			Err(error) => {
				match error {
					ErrorGetObjectWithCheckAccess::ObjectNotFound => {
						error!("objects not found {}", self.global_object_id);
					}
					ErrorGetObjectWithCheckAccess::AccessNotAllowed => {
						error!("client has access to delete objects {}", self.global_object_id)
					}
				}
			}
		}
	}
}

