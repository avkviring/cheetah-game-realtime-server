use cheetah_relay_common::commands::command::C2SCommand;

use crate::ffi::execute_with_client;

pub mod long_value;

#[allow(unused_must_use)]
fn send_command(command: C2SCommand) {
	execute_with_client(|client| { client.send(command); });
}