use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::commands::command::float_counter::{IncrementFloat64C2SCommand, SetFloat64Command};
use cheetah_relay_common::constants::FieldID;

use crate::ffi::{execute_with_client, GameObjectIdFFI};
use crate::ffi::command::send_command;

#[no_mangle]
#[allow(unused_must_use)]
pub extern fn register_float_value_listener(listener: extern fn(&GameObjectIdFFI, FieldID, f64)) {
	execute_with_client(|client| {
		client.register_float_value_listener(listener);
	});
}

#[no_mangle]
pub extern "C" fn set_float_value(object_id: &GameObjectIdFFI, field_id: FieldID, value: f64) {
	send_command(C2SCommand::SetFloatValue(SetFloat64Command { object_id: From::from(object_id), field_id, value }));
}

#[no_mangle]
pub extern "C" fn inc_float_value(object_id: &GameObjectIdFFI, field_id: FieldID, increment: f64) {
	send_command(C2SCommand::IncrementFloatCounter(IncrementFloat64C2SCommand { object_id: From::from(object_id), field_id, increment }));
}
