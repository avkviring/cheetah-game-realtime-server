use cheetah_matches_realtime_common::commands::binary_value::BinaryValue;
use cheetah_matches_realtime_common::commands::c2s::C2SCommand;
use cheetah_matches_realtime_common::commands::types::field::SetFieldCommand;
use cheetah_matches_realtime_common::commands::types::structure::CompareAndSetStructureCommand;
use cheetah_matches_realtime_common::constants::FieldId;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, BufferFFI, GameObjectIdFFI};

#[no_mangle]
#[allow(unused_must_use)]
pub extern "C" fn set_structure_listener(client_id: ClientId, listener: extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, &BufferFFI)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_structure = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn set_structure(client_id: ClientId, object_id: &GameObjectIdFFI, field_id: FieldId, structure: &BufferFFI) -> u8 {
	send_command(
		client_id,
		C2SCommand::SetField(SetFieldCommand {
			object_id: object_id.into(),
			field_id,
			value: BinaryValue::from(structure).as_slice().into(),
		}),
	)
}

#[no_mangle]
pub extern "C" fn compare_and_set_structure(
	client_id: ClientId,
	object_id: &GameObjectIdFFI,
	field_id: FieldId,
	current: &BufferFFI,
	new: &BufferFFI,
	has_reset: bool,
	reset: &BufferFFI,
) -> u8 {
	send_command(
		client_id,
		C2SCommand::CompareAndSetStructure(CompareAndSetStructureCommand {
			current: current.into(),
			field_id,
			new: new.into(),
			object_id: object_id.into(),
			reset: if has_reset { Some(reset.into()) } else { None },
		}),
	)
}
