use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::types::field::DeleteField;
use cheetah_common::room::field::{FieldId, FieldType};
use cheetah_common::room::object::GameObjectId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;

#[no_mangle]
pub extern "C" fn delete_field(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, field_type: FieldType) -> u8 {
	send_command(
		client_id,
		C2SCommand::DeleteField(DeleteField {
			object_id: *object_id,
			field_id,
			field_type,
		}),
	)
}
