use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::long::{IncrementLongC2SCommand, SetLongCommand};
use cheetah_common::room::field::{Field, FieldType};
use cheetah_protocol::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

impl ServerCommandExecutor for IncrementLongC2SCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let action = |object: &mut GameObject| {
			let current = object.longs.get(self.field_id).cloned().unwrap_or_default();
			let result = match current.checked_add(self.increment) {
				None => {
					tracing::error!("[IncrementLongC2SCommand] overflow, current({:?}) increment({:?})", current, self.increment);
					current
				}
				Some(result) => {
					object.longs.set(self.field_id, result);
					result
				}
			};

			Ok(Some(S2CCommand::SetLong(SetLongCommand {
				object_id: self.object_id,
				field_id: self.field_id,
				value: result,
			})))
		};

		room.send_command_from_action(
			self.object_id,
			Field {
				id: self.field_id,
				field_type: FieldType::Long,
			},
			member_id,
			None,
			action,
		)
	}
}

impl ServerCommandExecutor for SetLongCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id;

		let action = |object: &mut GameObject| {
			object.longs.set(self.field_id, self.value.clone());
			Ok(Some(S2CCommand::SetLong(self.clone())))
		};

		room.send_command_from_action(
			object_id,
			Field {
				id: field_id,
				field_type: FieldType::Long,
			},
			member_id,
			None,
			action,
		)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::long::{IncrementLongC2SCommand, SetLongCommand};
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::field::FieldId;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;
	use cheetah_protocol::RoomMemberId;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::MemberTemplate;
	use crate::room::template::config::RoomTemplate;
	use crate::room::Room;

	const FIELD_ID: FieldId = 100;

	#[test]
	fn should_set_long_command() {
		let (mut room, member_id, object_id) = setup();

		room.test_out_commands.clear();
		let command = SetLongCommand {
			object_id,
			field_id: FIELD_ID,
			value: 100,
		};
		command.execute(&mut room, member_id).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		assert_eq!(*object.longs.get(FIELD_ID).unwrap(), 100);
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if
			c==command));
	}

	#[test]
	fn should_increment_long_command() {
		let (mut room, member_id, object_id) = setup();

		room.test_out_commands.clear();
		let command = IncrementLongC2SCommand {
			object_id,
			field_id: FIELD_ID,
			increment: 100,
		};
		command.clone().execute(&mut room, member_id).unwrap();
		command.execute(&mut room, member_id).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		assert_eq!(*object.longs.get(FIELD_ID).unwrap(), 200);

		let result = SetLongCommand {
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
		let command = IncrementLongC2SCommand {
			object_id,
			field_id: FIELD_ID,
			increment: i64::MAX,
		};
		command.clone().execute(&mut room, member_id).unwrap();
		command.execute(&mut room, member_id).unwrap();
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
