use cheetah_common::commands::c2s::C2SCommand;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::execute_with_client;

#[no_mangle]
pub extern "C" fn attach_to_room(client_id: ClientId) -> u8 {
	execute_with_client(client_id, |client| Ok(client.attach_to_room()?))
}

#[no_mangle]
pub extern "C" fn detach_from_room(client_id: ClientId) -> u8 {
	send_command(client_id, C2SCommand::DetachFromRoom)
}
