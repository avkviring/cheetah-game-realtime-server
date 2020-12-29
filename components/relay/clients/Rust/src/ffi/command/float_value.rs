use cheetah_relay_common::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};

use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::constants::FieldId;

use crate::ffi::command::{send_command, S2CMetaCommandInformationFFI};
use crate::ffi::{execute_with_client, GameObjectIdFFI};

#[no_mangle]
pub extern "C" fn set_float_value_listener(listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, f64)) -> bool {
	execute_with_client(|client| {
		client.register_float_value_listener(listener);
	})
	.is_ok()
}

#[no_mangle]
pub extern "C" fn set_float_value(object_id: &GameObjectIdFFI, field_id: FieldId, value: f64) -> bool {
	send_command(C2SCommand::SetFloat(SetFloat64Command {
		object_id: From::from(object_id),
		field_id,
		value,
	}))
}

#[no_mangle]
pub extern "C" fn inc_float_value(object_id: &GameObjectIdFFI, field_id: FieldId, increment: f64) -> bool {
	send_command(C2SCommand::IncrementFloatCounter(IncrementFloat64C2SCommand {
		object_id: From::from(object_id),
		field_id,
		increment,
	}))
}
