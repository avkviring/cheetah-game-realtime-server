use cheetah_matches_relay_common::commands::c2s::C2SCommand;

use crate::clients::registry::ClientId;
use crate::ffi::execute_with_client;

pub mod event;
pub mod field;
pub mod float_value;
pub mod long_value;
pub mod object;
pub mod room;
pub mod structure;

fn send_command(client_id: ClientId, command: C2SCommand) -> u8 {
	execute_with_client(client_id, |client| Ok(client.send(command)?))
}
