use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;

use crate::ffi::{execute_with_client, GameObjectIdFFI};
use crate::ffi::command::send_command;

#[no_mangle]
pub extern fn set_delete_object_listener(listener: extern fn(&S2CMetaCommandInformation, &GameObjectIdFFI)) -> bool {
	execute_with_client(|client| {
		client.register_delete_object_listener(listener);
	}).is_ok()
}

#[no_mangle]
pub extern "C" fn delete_object(object_id: &GameObjectIdFFI) -> bool {
	send_command(C2SCommand::Delete(DeleteGameObjectCommand { object_id: From::from(object_id) }))
}
