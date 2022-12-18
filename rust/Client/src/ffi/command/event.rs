use cheetah_common::commands::binary_value::BinaryValue;
use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::field::FieldId;
use cheetah_common::commands::types::event::{EventCommand, TargetEventCommand};
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::RoomMemberId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::execute_with_client;

#[no_mangle]
pub extern "C" fn set_event_listener(client_id: ClientId, listener: extern "C" fn(RoomMemberId, &GameObjectId, FieldId, &BinaryValue)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_event = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn send_event(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, event: &BinaryValue) -> u8 {
	send_command(
		client_id,
		C2SCommand::Event(EventCommand {
			object_id: *object_id,
			field_id,
			event: *event,
		}),
	)
}

#[no_mangle]
pub extern "C" fn send_target_event(
	client_id: ClientId,
	target_member_id: RoomMemberId,
	object_id: &GameObjectId,
	field_id: FieldId,
	event: &BinaryValue,
) -> u8 {
	let event_command = EventCommand {
		object_id: *object_id,
		field_id,
		event: *event,
	};
	send_command(
		client_id,
		C2SCommand::TargetEvent(TargetEventCommand {
			target: target_member_id,
			event: event_command,
		}),
	)
}
