use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
use cheetah_matches_realtime_common::commands::types::field::SetFieldCommand;
use cheetah_matches_realtime_common::commands::types::long::IncrementLongC2SCommand;
use cheetah_matches_realtime_common::commands::FieldType;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::{Field, GameObject};
use crate::room::template::config::Permission;
use crate::room::Room;

impl ServerCommandExecutor for IncrementLongC2SCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let action = |object: &mut GameObject| {
			let value = match object.get_field::<i64>(self.field_id) {
				Some(value) => match (*value).checked_add(self.increment) {
					None => {
						tracing::error!(
							"[IncrementLongC2SCommand] overflow, current({:?}) increment({:?})",
							value,
							self.increment
						);
						*value
					}
					Some(result) => {
						object.set_field(self.field_id, result)?;
						result
					}
				},
				None => {
					object.set_field(self.field_id, self.increment)?;
					self.increment
				}
			};
			Ok(Some(S2CCommand::SetField(SetFieldCommand {
				object_id: self.object_id.clone(),
				field_id: self.field_id,
				value: value.into(),
			})))
		};
		room.send_command_from_action(
			&self.object_id,
			Field {
				id: self.field_id,
				field_type: FieldType::Long,
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		)
	}
}

impl ServerCommandExecutor for SetFieldCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();

		let action = |object: &mut GameObject| {
			object.set_field_wrapped(self.field_id, self.value.to_owned())?;
			Ok(Some(S2CCommand::SetField(self.clone())))
		};

		room.send_command_from_action(
			&object_id,
			Field {
				id: field_id,
				field_type: self.value.field_type(),
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
	use cheetah_matches_realtime_common::commands::types::field::SetFieldCommand;
	use cheetah_matches_realtime_common::commands::types::long::IncrementLongC2SCommand;
	use cheetah_matches_realtime_common::room::access::AccessGroups;
	use cheetah_matches_realtime_common::room::object::GameObjectId;
	use cheetah_matches_realtime_common::room::owner::GameObjectOwner;
	use cheetah_matches_realtime_common::room::RoomMemberId;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::MemberTemplate;
	use crate::room::template::config::RoomTemplate;
	use crate::room::Room;

	#[test]
	fn should_set_long_command() {
		let (mut room, user, object_id) = setup();

		room.test_out_commands.clear();
		let command = SetFieldCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100.into(),
		};
		command.execute(&mut room, user).unwrap();

		let object = room.get_object(&object_id).unwrap();
		assert_eq!(*object.get_field::<i64>(10).unwrap(), 100);
		assert!(
			matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetField(c))) if c==command)
		);
	}

	#[test]
	fn should_increment_long_command() {
		let (mut room, user, object_id) = setup();

		room.test_out_commands.clear();
		let command = IncrementLongC2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: 100,
		};
		command.clone().execute(&mut room, user).unwrap();
		command.execute(&mut room, user).unwrap();

		let object = room.get_object(&object_id).unwrap();
		assert_eq!(*object.get_field::<i64>(10).unwrap(), 200);

		let result = SetFieldCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 200.into(),
		};

		room.test_out_commands.pop_back();
		assert!(
			matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetField(c))) if c==result)
		);
	}

	#[test]
	fn should_not_panic_if_overflow() {
		let (mut room, user, object_id) = setup();
		room.test_out_commands.clear();
		let command = IncrementLongC2SCommand {
			object_id,
			field_id: 10,
			increment: i64::MAX,
		};
		command.clone().execute(&mut room, user).unwrap();
		command.execute(&mut room, user).unwrap();
	}

	fn setup() -> (Room, RoomMemberId, GameObjectId) {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::from_template(template);
		let user_id = room.register_member(MemberTemplate::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(
			GameObjectOwner::Member(user_id),
			access_groups,
		);
		object.created = true;
		let object_id = object.id.clone();
		(room, user_id, object_id)
	}
}
