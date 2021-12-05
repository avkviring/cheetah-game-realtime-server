use cheetah_matches_relay_common::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
use cheetah_matches_relay_common::commands::command::C2SCommand;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, GameObjectIdFFI};
use crate::registry::ClientId;

#[no_mangle]
pub extern "C" fn set_double_value_listener(
	client_id: ClientId,
	listener: extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, f64),
) -> bool {
	execute_with_client(client_id, |client| client.register_float_value_listener(listener)).is_ok()
}

#[no_mangle]
pub extern "C" fn set_double_value(client_id: ClientId, object_id: &GameObjectIdFFI, field_id: FieldId, value: f64) -> bool {
	send_command(
		client_id,
		C2SCommand::SetFloat(SetFloat64Command {
			object_id: From::from(object_id),
			field_id,
			value,
		}),
	)
}

#[no_mangle]
pub extern "C" fn inc_double_value(client_id: ClientId, object_id: &GameObjectIdFFI, field_id: FieldId, increment: f64) -> bool {
	send_command(
		client_id,
		C2SCommand::IncrementFloatCounter(IncrementFloat64C2SCommand {
			object_id: From::from(object_id),
			field_id,
			increment,
		}),
	)
}
