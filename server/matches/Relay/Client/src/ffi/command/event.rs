use cheetah_matches_relay_common::commands::command::event::{EventCommand, TargetEventCommand};
use cheetah_matches_relay_common::commands::command::C2SCommand;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, BufferFFI, GameObjectIdFFI};
use crate::registry::ClientId;

#[no_mangle]
pub extern "C" fn set_event_listener(
	client_id: ClientId,
	listener: extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, &BufferFFI),
) -> bool {
	execute_with_client(client_id, |client| client.register_event_listener(listener)).is_ok()
}

#[no_mangle]
pub extern "C" fn send_event(client_id: ClientId, object_id: &GameObjectIdFFI, field_id: FieldId, event: &BufferFFI) -> bool {
	send_command(
		client_id,
		C2SCommand::Event(EventCommand {
			object_id: From::from(object_id),
			field_id,
			event: From::from(event),
		}),
	)
}

#[no_mangle]
pub extern "C" fn send_target_event(
	client_id: ClientId,
	target_user: RoomMemberId,
	object_id: &GameObjectIdFFI,
	field_id: FieldId,
	event: &BufferFFI,
) -> bool {
	let event_command = EventCommand {
		object_id: From::from(object_id),
		field_id,
		event: From::from(event),
	};
	send_command(
		client_id,
		C2SCommand::TargetEvent(TargetEventCommand {
			target: target_user,
			event: event_command,
		}),
	)
}
