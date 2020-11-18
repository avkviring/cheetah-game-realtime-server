use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::commands::command::event::EventCommand;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::constants::FieldID;

use crate::ffi::{BufferFFI, execute_with_client, GameObjectIdFFI};
use crate::ffi::command::send_command;

#[no_mangle]
pub extern fn set_event_listener(listener: extern fn(&S2CMetaCommandInformation, &GameObjectIdFFI, FieldID, &BufferFFI)) -> bool {
	execute_with_client(|client| {
		client.register_event_listener(listener);
	}).is_ok()
}

#[no_mangle]
pub extern "C" fn send_event(object_id: &GameObjectIdFFI, field_id: FieldID, event: &BufferFFI) -> bool {
	send_command(C2SCommand::Event(EventCommand { object_id: From::from(object_id), field_id, event: From::from(event) }))
}
