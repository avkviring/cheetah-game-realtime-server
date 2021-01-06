use cheetah_relay_common::commands::command::C2SCommand;

use crate::ffi::command::send_command;
use crate::ffi::execute_with_client;

#[no_mangle]
pub extern "C" fn attach_to_room() -> bool {
	execute_with_client(|client| {
		client.attach_to_room();
	})
	.is_ok()
}

#[no_mangle]
pub extern "C" fn detach_from_room() -> bool {
	send_command(C2SCommand::DetachFromRoom)
}
