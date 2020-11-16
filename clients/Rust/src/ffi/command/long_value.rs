use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::commands::command::long_counter::{IncrementLongC2SCommand, SetLongCommand};
use cheetah_relay_common::constants::FieldID;


use crate::ffi::command::send_command;
use crate::ffi::{GameObjectIdFFI, execute_with_client};

#[no_mangle]
#[allow(unused_must_use)]
pub extern fn register_long_value_listener(listener: extern fn(GameObjectIdFFI, FieldID, i64)) {
	execute_with_client(|client|{
		client.register_long_value_listener(listener);
	});
}

#[no_mangle]
pub extern "C" fn set_long_value(object_id: &GameObjectIdFFI, field_id: FieldID, value: i64) {
	send_command(C2SCommand::SetLongValue(SetLongCommand { object_id: From::from(object_id), field_id, value }));
}

#[no_mangle]
pub extern "C" fn inc_long_value(object_id: &GameObjectIdFFI, field_id: FieldID, increment: i64) {
	send_command(C2SCommand::IncrementLongValue(IncrementLongC2SCommand { object_id: From::from(object_id), field_id, increment }));
}
