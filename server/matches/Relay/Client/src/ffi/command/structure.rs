use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::commands::types::structure::StructureCommand;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, BufferFFI, GameObjectIdFFI};
use crate::registry::ClientId;

#[no_mangle]
#[allow(unused_must_use)]
pub extern "C" fn set_structure_listener(
	client_id: ClientId,
	listener: extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, &BufferFFI),
) -> bool {
	execute_with_client(client_id, |client| client.register_structure_listener(listener)).is_ok()
}

#[no_mangle]
pub extern "C" fn set_structure(
	client_id: ClientId,
	object_id: &GameObjectIdFFI,
	field_id: FieldId,
	structure: &BufferFFI,
) -> bool {
	send_command(
		client_id,
		C2SCommand::SetStruct(StructureCommand {
			object_id: From::from(object_id),
			field_id,
			structure: From::from(structure),
		}),
	)
}
