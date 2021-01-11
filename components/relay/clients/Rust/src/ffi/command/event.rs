use cheetah_relay_common::commands::command::event::EventCommand;
use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::constants::FieldId;

use crate::ffi::command::{send_command, S2CMetaCommandInformationFFI};
use crate::ffi::{execute_with_client, BufferFFI, GameObjectIdFFI};

#[no_mangle]
pub extern "C" fn set_event_listener(listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, &BufferFFI)) -> bool {
	execute_with_client(|client, trace| {
		(
			{
				client.register_event_listener(listener);
			},
			if trace {
				listener(&S2CMetaCommandInformationFFI::stub(), &GameObjectIdFFI::stub(), 77, &BufferFFI::stub());
				Some(format!("set_event_listener"))
			} else {
				None
			},
		)
	})
	.is_ok()
}

#[no_mangle]
pub extern "C" fn send_event(object_id: &GameObjectIdFFI, field_id: FieldId, event: &BufferFFI) -> bool {
	send_command(C2SCommand::Event(EventCommand {
		object_id: From::from(object_id),
		field_id,
		event: From::from(event),
	}))
}
