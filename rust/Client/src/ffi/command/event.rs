use crate::clients::registry::ClientId;
use crate::ffi::command::{send_command, BufferFFI};
use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::types::event::TargetEvent;
use cheetah_common::commands::types::structure::BinaryField;
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::GameObjectId;
use cheetah_game_realtime_protocol::RoomMemberId;

#[no_mangle]
pub extern "C" fn send_event(client_id: ClientId, object_id: &GameObjectId, field_id: FieldId, event: &BufferFFI) -> u8 {
	send_command(
		client_id,
		C2SCommand::Event(
			BinaryField {
				object_id: *object_id,
				field_id,
				value: event.into(),
			}
			.into(),
		),
	)
}

#[no_mangle]
pub extern "C" fn send_target_event(client_id: ClientId, target_member_id: RoomMemberId, object_id: &GameObjectId, field_id: FieldId, event: &BufferFFI) -> u8 {
	let event_command = BinaryField {
		object_id: *object_id,
		field_id,
		value: event.into(),
	};
	send_command(
		client_id,
		C2SCommand::TargetEvent(
			TargetEvent {
				target: target_member_id,
				event: event_command,
			}
			.into(),
		),
	)
}
