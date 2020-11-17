use cheetah_relay_common::commands::command::C2SCommand;

use crate::ffi::command::send_command;

#[no_mangle]
pub extern "C" fn load_room() {
	send_command(C2SCommand::LoadRoom);
}