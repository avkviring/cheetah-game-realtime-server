use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::field::FieldId;
use cheetah_common::commands::types::field::SetFieldCommand;
use cheetah_common::commands::types::float::IncrementDoubleC2SCommand;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::RoomMemberId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::execute_with_client;

#[no_mangle]
pub extern "C" fn set_double_value_listener(client_id: ClientId, listener: extern "C" fn(RoomMemberId, &GameObjectId, FieldId, f64)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_float_value = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn set_double_value(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, value: f64) -> u8 {
	send_command(
		client_id,
		C2SCommand::SetField(SetFieldCommand {
			object_id: *object_id,
			field_id,
			value: value.into(),
		}),
	)
}

#[no_mangle]
pub extern "C" fn inc_double_value(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, increment: f64) -> u8 {
	send_command(
		client_id,
		C2SCommand::IncrementDouble(IncrementDoubleC2SCommand {
			object_id: *object_id,
			field_id,
			increment,
		}),
	)
}
