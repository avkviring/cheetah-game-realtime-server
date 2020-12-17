use cheetah_relay_common::commands::command::event::EventCommand;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::constants::FieldIdType;

use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, BufferFFI, GameObjectIdFFI};

#[no_mangle]
pub extern "C" fn set_event_listener(listener: extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, FieldIdType, &BufferFFI)) -> bool {
	execute_with_client(|client| {
		client.register_event_listener(listener);
	})
	.is_ok()
}

#[no_mangle]
pub extern "C" fn send_event(object_id: &GameObjectIdFFI, field_id: FieldIdType, event: &BufferFFI) -> bool {
	send_command(C2SCommand::Event(EventCommand {
		object_id: From::from(object_id),
		field_id,
		event: From::from(event),
	}))
}
