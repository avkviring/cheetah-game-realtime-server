use cheetah_relay_common::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::constants::FieldID;

use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, GameObjectIdFFI};

#[no_mangle]
pub extern "C" fn set_float_value_listener(listener: extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, FieldID, f64)) -> bool {
	execute_with_client(|client| {
		client.register_float_value_listener(listener);
	})
	.is_ok()
}

#[no_mangle]
pub extern "C" fn set_float_value(object_id: &GameObjectIdFFI, field_id: FieldID, value: f64) -> bool {
	send_command(C2SCommand::SetFloat(SetFloat64Command {
		object_id: From::from(object_id),
		field_id,
		value,
	}))
}

#[no_mangle]
pub extern "C" fn inc_float_value(object_id: &GameObjectIdFFI, field_id: FieldID, increment: f64) -> bool {
	send_command(C2SCommand::IncrementFloatCounter(IncrementFloat64C2SCommand {
		object_id: From::from(object_id),
		field_id,
		increment,
	}))
}
