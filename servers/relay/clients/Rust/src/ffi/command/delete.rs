use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
use cheetah_relay_common::commands::command::C2SCommand;

use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, GameObjectIdFFI};

#[no_mangle]
pub extern "C" fn set_delete_object_listener(listener: extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI)) -> bool {
	execute_with_client(|client| {
		client.register_delete_object_listener(listener);
	})
	.is_ok()
}

#[no_mangle]
pub extern "C" fn delete_object(object_id: &GameObjectIdFFI) -> bool {
	send_command(C2SCommand::Delete(DeleteGameObjectCommand {
		object_id: From::from(object_id),
	}))
}
