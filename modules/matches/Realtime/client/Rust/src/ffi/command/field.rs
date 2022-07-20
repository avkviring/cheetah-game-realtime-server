use cheetah_matches_realtime_common::commands::c2s::C2SCommand;
use cheetah_matches_realtime_common::commands::types::field::DeleteFieldCommand;
use cheetah_matches_realtime_common::constants::FieldId;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, FieldTypeFFI, GameObjectIdFFI};

#[no_mangle]
pub extern "C" fn set_delete_field_listener(
	client_id: ClientId,
	listener: extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, FieldTypeFFI),
) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_delete_field = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn delete_field(
	client_id: ClientId,
	object_id: &GameObjectIdFFI,
	field_id: FieldId,
	field_type: FieldTypeFFI,
) -> u8 {
	send_command(
		client_id,
		C2SCommand::DeleteField(DeleteFieldCommand {
			object_id: From::from(object_id),
			field_id,
			field_type: From::from(field_type),
		}),
	)
}
