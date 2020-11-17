use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;

use crate::ffi::{execute_with_client, GameObjectIdFFI};
use crate::ffi::command::send_command;

#[no_mangle]
#[allow(unused_must_use)]
pub extern fn register_delete_listener(listener: extern fn(&GameObjectIdFFI)) {
	execute_with_client(|client| {
		client.register_delete_object_listener(listener);
	});
}

#[no_mangle]
pub extern "C" fn delete(object_id: &GameObjectIdFFI) {
	send_command(C2SCommand::Delete(DeleteGameObjectCommand { object_id: From::from(object_id) }));
}
