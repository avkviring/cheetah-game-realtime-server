use cheetah_matches_relay_common::commands::command::long::{CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand};
use cheetah_matches_relay_common::commands::command::C2SCommand;
use cheetah_matches_relay_common::constants::FieldId;

use crate::ffi::command::{send_command, S2CMetaCommandInformationFFI};
use crate::ffi::{execute_with_client, GameObjectIdFFI};

#[no_mangle]
pub extern "C" fn set_long_value_listener(
	listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, i64),
) -> bool {
	execute_with_client(|client| client.register_long_value_listener(listener)).is_ok()
}

#[no_mangle]
pub extern "C" fn set_long_value(object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) -> bool {
	send_command(C2SCommand::SetLong(SetLongCommand {
		object_id: From::from(object_id),
		field_id,
		value,
	}))
}

#[no_mangle]
pub extern "C" fn inc_long_value(object_id: &GameObjectIdFFI, field_id: FieldId, increment: i64) -> bool {
	send_command(C2SCommand::IncrementLongValue(IncrementLongC2SCommand {
		object_id: From::from(object_id),
		field_id,
		increment,
	}))
}

#[no_mangle]
pub extern "C" fn compare_and_set_long_value(
	object_id: &GameObjectIdFFI,
	field_id: FieldId,
	current: i64,
	new: i64,
	reset: i64,
) -> bool {
	send_command(C2SCommand::CompareAndSetLongValue(CompareAndSetLongCommand {
		object_id: From::from(object_id),
		field_id,
		current,
		new,
		reset,
	}))
}
