use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::field::FieldId;
use cheetah_common::commands::types::field::DeleteFieldCommand;
use cheetah_common::commands::FieldType;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::RoomMemberId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::execute_with_client;

#[no_mangle]
pub extern "C" fn set_delete_field_listener(client_id: ClientId, listener: extern "C" fn(RoomMemberId, &GameObjectId, FieldId, FieldType)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_delete_field = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn delete_field(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, field_type: FieldType) -> u8 {
	send_command(
		client_id,
		C2SCommand::DeleteField(DeleteFieldCommand {
			object_id: *object_id,
			field_id,
			field_type: From::from(field_type),
		}),
	)
}
