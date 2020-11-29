use cheetah_relay_common::commands::command::C2SCommand;

use crate::ffi::execute_with_client;

pub mod create;
pub mod delete;
pub mod event;
pub mod float_value;
pub mod long_value;
pub mod room;
pub mod structure;

fn send_command(command: C2SCommand) -> bool {
	execute_with_client(|client| {
		client.send(command);
	})
	.is_ok()
}
