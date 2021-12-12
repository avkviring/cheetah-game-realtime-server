use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand};
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, GameObjectIdFFI};
use crate::registry::ClientId;

#[no_mangle]
pub extern "C" fn set_long_value_listener(
	client_id: ClientId,
	listener: extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, i64),
) -> bool {
	execute_with_client(client_id, |client| client.register_long_value_listener(listener)).is_ok()
}

#[no_mangle]
pub extern "C" fn set_long_value(client_id: ClientId, object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) -> bool {
	send_command(
		client_id,
		C2SCommand::SetLong(SetLongCommand {
			object_id: From::from(object_id),
			field_id,
			value,
		}),
	)
}

#[no_mangle]
pub extern "C" fn inc_long_value(client_id: ClientId, object_id: &GameObjectIdFFI, field_id: FieldId, increment: i64) -> bool {
	send_command(
		client_id,
		C2SCommand::IncrementLongValue(IncrementLongC2SCommand {
			object_id: From::from(object_id),
			field_id,
			increment,
		}),
	)
}

#[no_mangle]
pub extern "C" fn compare_and_set_long_value(
	client_id: ClientId,
	object_id: &GameObjectIdFFI,
	field_id: FieldId,
	current: i64,
	new: i64,
	reset: i64,
) -> bool {
	send_command(
		client_id,
		C2SCommand::CompareAndSetLongValue(CompareAndSetLongCommand {
			object_id: From::from(object_id),
			field_id,
			current,
			new,
			reset,
		}),
	)
}
