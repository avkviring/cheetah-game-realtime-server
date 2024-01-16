use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::types::float::{DoubleField, IncrementDouble};
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::GameObjectId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;

#[no_mangle]
pub extern "C" fn set_double_value(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, value: f64) -> u8 {
	send_command(
		client_id,
		C2SCommand::SetDouble(DoubleField {
			object_id: *object_id,
			field_id,
			value,
		}),
	)
}

#[no_mangle]
pub extern "C" fn inc_double_value(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, increment: f64) -> u8 {
	send_command(
		client_id,
		C2SCommand::IncrementDouble(IncrementDouble {
			object_id: *object_id,
			field_id,
			increment,
		}),
	)
}
