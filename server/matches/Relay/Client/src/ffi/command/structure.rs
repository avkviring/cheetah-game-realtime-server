use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::commands::types::structure::SetStructureCommand;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, BufferFFI, GameObjectIdFFI};

#[no_mangle]
#[allow(unused_must_use)]
pub extern "C" fn set_structure_listener(
	client_id: ClientId,
	listener: extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, &BufferFFI),
) -> u8 {
	execute_with_client(client_id, |client| Ok(client.listener_structure = Some(listener)))
}

#[no_mangle]
pub extern "C" fn set_structure(
	client_id: ClientId,
	object_id: &GameObjectIdFFI,
	field_id: FieldId,
	structure: &BufferFFI,
) -> u8 {
	send_command(
		client_id,
		C2SCommand::SetStructure(SetStructureCommand {
			object_id: From::from(object_id),
			field_id,
			structure: From::from(structure),
		}),
	)
}
