use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::types::structure::SetStructureCommand;
use cheetah_common::room::buffer::Buffer;
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::GameObjectId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;

#[no_mangle]
pub extern "C" fn set_structure(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, structure: &Buffer) -> u8 {
	send_command(
		client_id,
		C2SCommand::SetStructure(SetStructureCommand {
			object_id: *object_id,
			field_id,
			value: *structure,
		}),
	)
}
