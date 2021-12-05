use cheetah_matches_relay_common::commands::command::C2SCommand;

use crate::ffi::execute_with_client;
use crate::registry::ClientId;

pub mod event;
pub mod float_value;
pub mod long_value;
pub mod object;
pub mod room;
pub mod structure;

fn send_command(client_id: ClientId, command: C2SCommand) -> bool {
	execute_with_client(client_id, |client| client.send(command)).is_ok()
}
