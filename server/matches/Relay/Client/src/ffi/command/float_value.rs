use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::commands::types::field::SetFieldCommand;
use cheetah_matches_relay_common::commands::types::float::IncrementDoubleC2SCommand;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, GameObjectIdFFI};

#[no_mangle]
pub extern "C" fn set_double_value_listener(
	client_id: ClientId,
	listener: extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, f64),
) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_float_value = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn set_double_value(client_id: ClientId, object_id: &GameObjectIdFFI, field_id: FieldId, value: f64) -> u8 {
	send_command(
		client_id,
		C2SCommand::SetDouble(SetFieldCommand {
			object_id: From::from(object_id),
			field_id,
			value: value.into(),
		}),
	)
}

#[no_mangle]
pub extern "C" fn inc_double_value(client_id: ClientId, object_id: &GameObjectIdFFI, field_id: FieldId, increment: f64) -> u8 {
	send_command(
		client_id,
		C2SCommand::IncrementDouble(IncrementDoubleC2SCommand {
			object_id: From::from(object_id),
			field_id,
			increment,
		}),
	)
}
