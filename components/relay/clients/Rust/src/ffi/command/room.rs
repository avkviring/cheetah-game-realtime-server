use cheetah_relay_common::commands::command::C2SCommand;

use crate::ffi::command::send_command;
use crate::ffi::execute_with_client;

#[no_mangle]
pub extern "C" fn attach_to_room() -> bool {
	execute_with_client(|client, trace| {
		(
			{
				client.attach_to_room();
			},
			if trace { Some(format!("attach_to_room")) } else { None },
		)
	})
	.is_ok()
}

#[no_mangle]
pub extern "C" fn detach_from_room() -> bool {
	send_command(C2SCommand::DetachFromRoom)
}
