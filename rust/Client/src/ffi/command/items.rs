use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::types::structure::BinaryField;
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::GameObjectId;

use crate::clients::registry::ClientId;
use crate::ffi::command::{send_command, BufferFFI};

#[no_mangle]
pub extern "C" fn add_item(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, structure: &BufferFFI) -> u8 {
	send_command(
		client_id,
		C2SCommand::AddItem(
			BinaryField {
				object_id: *object_id,
				field_id,
				value: structure.into(),
			}
			.into(),
		),
	)
}
