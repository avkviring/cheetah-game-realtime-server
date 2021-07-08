use cheetah_matches_relay_common::commands::command::structure::StructureCommand;
use cheetah_matches_relay_common::commands::command::C2SCommand;
use cheetah_matches_relay_common::constants::FieldId;

use crate::ffi::command::{send_command, S2CMetaCommandInformationFFI};
use crate::ffi::{execute_with_client, BufferFFI, GameObjectIdFFI};

#[no_mangle]
#[allow(unused_must_use)]
pub extern "C" fn set_structure_listener(listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, &BufferFFI)) -> bool {
	execute_with_client(|client, trace| {
		(
			{
				client.register_structure_listener(listener);
			},
			if trace {
				listener(&S2CMetaCommandInformationFFI::stub(), &GameObjectIdFFI::stub(), 77, &BufferFFI::stub());
				Some(format!("set_structure_listener"))
			} else {
				None
			},
		)
	})
	.is_ok()
}

#[no_mangle]
pub extern "C" fn set_structure(object_id: &GameObjectIdFFI, field_id: FieldId, structure: &BufferFFI) -> bool {
	send_command(C2SCommand::SetStruct(StructureCommand {
		object_id: From::from(object_id),
		field_id,
		structure: From::from(structure),
	}))
}