use cheetah_game_realtime_protocol::RoomMemberId;

use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::long::{IncrementLong, LongField};

use crate::server::room::command::ServerCommandError;
use crate::server::room::object::GameObject;
use crate::server::room::Room;

pub(crate) fn increment(increment_long: &IncrementLong, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let action = |object: &mut GameObject| {
		let current = object.long_fields.get(increment_long.field_id).cloned().unwrap_or_default();
		let result = match current.checked_add(increment_long.increment) {
			None => {
				tracing::error!("[IncrementLongC2SCommand] overflow, current({:?}) increment({:?})", current, increment_long.increment);
				current
			}
			Some(result) => {
				object.long_fields.set(increment_long.field_id, result);
				result
			}
		};

		Ok(Some(S2CCommand::SetLong(LongField {
			object_id: increment_long.object_id,
			field_id: increment_long.field_id,
			value: result,
		})))
	};

	room.send_command_from_action(increment_long.object_id, member_id, None, action)
}

pub(crate) fn set(long_field: &LongField, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let object_id = long_field.object_id;

	let action = |object: &mut GameObject| {
		object.long_fields.set(long_field.field_id, long_field.value);
		Ok(Some(S2CCommand::SetLong(*long_field)))
	};

	room.send_command_from_action(object_id, member_id, None, action)
}

#[cfg(test)]
mod tests {
	use cheetah_game_realtime_protocol::RoomMemberId;

	use crate::server::room::command::long::{increment, set};
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::long::{IncrementLong, LongField};
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::field::FieldId;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::server::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::server::room::Room;

	const FIELD_ID: FieldId = 100;

	#[test]
	fn should_set_long_command() {
		let (mut room, member_id, object_id) = setup();

		room.test_out_commands.clear();
		let command = LongField {
			object_id,
			field_id: FIELD_ID,
			value: 100,
		};
		set(&command, &mut room, member_id).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		assert_eq!(*object.long_fields.get(FIELD_ID).unwrap(), 100);
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if
			c==command));
	}

	#[test]
	fn should_increment_long_command() {
		let (mut room, member_id, object_id) = setup();

		room.test_out_commands.clear();
		let command = IncrementLong {
			object_id,
			field_id: FIELD_ID,
			increment: 100,
		};
		increment(&command, &mut room, member_id).unwrap();
		increment(&command, &mut room, member_id).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		assert_eq!(*object.long_fields.get(FIELD_ID).unwrap(), 200);

		let result = LongField {
			object_id,
			field_id: FIELD_ID,
			value: 200,
		};

		room.test_out_commands.pop_back();
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if
			c==result));
	}

	#[test]
	fn should_not_panic_if_overflow() {
		let (mut room, member_id, object_id) = setup();
		room.test_out_commands.clear();
		let command = IncrementLong {
			object_id,
			field_id: FIELD_ID,
			increment: i64::MAX,
		};

		increment(&command, &mut room, member_id).unwrap();
		increment(&command, &mut room, member_id).unwrap();
	}

	fn setup() -> (Room, RoomMemberId, GameObjectId) {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::from_template(template);
		let member_id = room.register_member(MemberTemplate::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups);
		object.created = true;
		let object_id = object.id;
		(room, member_id, object_id)
	}
}
