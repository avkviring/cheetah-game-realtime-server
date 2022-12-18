use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::field::FieldId;
use cheetah_common::commands::types::field::SetFieldCommand;
use cheetah_common::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand};
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::RoomMemberId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::execute_with_client;

#[no_mangle]
pub extern "C" fn set_long_value_listener(client_id: ClientId, listener: extern "C" fn(RoomMemberId, &GameObjectId, FieldId, i64)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_long_value = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn set_long_value(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, value: i64) -> u8 {
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
pub extern "C" fn inc_long_value(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, increment: i64) -> u8 {
	send_command(
		client_id,
		C2SCommand::IncrementLongValue(IncrementLongC2SCommand {
			object_id: *object_id,
			field_id,
			increment,
		}),
	)
}

#[no_mangle]
pub extern "C" fn compare_and_set_long_value(
	client_id: ClientId,
	object_id: &GameObjectId,
	field_id: FieldId,
	current: i64,
	new: i64,
	has_reset: bool,
	reset: i64,
) -> u8 {
	send_command(
		client_id,
		C2SCommand::CompareAndSetLong(CompareAndSetLongCommand {
			object_id: *object_id,
			field_id,
			current,
			new,
			reset: has_reset.then_some(reset),
		}),
	)
}
