use cheetah_matches_relay_common::commands::command::C2SCommand;

use crate::ffi::command::send_command;
use crate::ffi::execute_with_client;
use crate::registry::ClientId;

#[no_mangle]
pub extern "C" fn attach_to_room(client_id: ClientId) -> bool {
	execute_with_client(client_id, |client| client.attach_to_room()).is_ok()
}

#[no_mangle]
pub extern "C" fn detach_from_room(client_id: ClientId) -> bool {
	send_command(client_id, C2SCommand::DetachFromRoom)
}
