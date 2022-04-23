use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::long::{IncrementLongC2SCommand, SetLongCommand};
use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::{CreateCommandsCollector, Field, GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::Permission;
use crate::room::Room;

impl ServerCommandExecutor for IncrementLongC2SCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let action = |object: &mut GameObject| {
			let value = match object.get_long(&self.field_id) {
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
						object.set_long(self.field_id, result)?;
						result
					}
				},
				None => {
					object.set_long(self.field_id, self.increment)?;
					self.increment
				}
			};
			Ok(Some(S2CCommand::SetLong(SetLongCommand {
				object_id: self.object_id.clone(),
				field_id: self.field_id,
				value,
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

impl ServerCommandExecutor for SetLongCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();

		let action = |object: &mut GameObject| {
			object.set_long(self.field_id, self.value)?;
			Ok(Some(S2CCommand::SetLong(self.clone())))
		};

		room.send_command_from_action(
			&object_id,
			Field {
				id: field_id,
				field_type: FieldType::Long,
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		)
	}
}
impl GameObject {
	pub fn longs_to_commands(&self, commands: &mut CreateCommandsCollector) -> Result<(), S2CommandWithFieldInfo> {
		for (field_id, v) in self.get_longs() {
			let command = S2CommandWithFieldInfo {
				field: Option::Some(Field {
					id: *field_id,
					field_type: FieldType::Long,
				}),
				command: S2CCommand::SetLong(SetLongCommand {
					object_id: self.id.clone(),
					field_id: *field_id,
					value: *v,
				}),
			};
			commands.push(command)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::s2c::S2CCommand;
	use cheetah_matches_relay_common::commands::types::long::{IncrementLongC2SCommand, SetLongCommand};
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;
	use cheetah_matches_relay_common::room::RoomMemberId;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::MemberTemplate;
	use crate::room::template::config::RoomTemplate;
	use crate::room::Room;

	#[test]
	fn should_set_long_command() {
		let (mut room, user, object_id) = setup();

		room.test_out_commands.clear();
		let command = SetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100,
		};
		command.execute(&mut room, user).unwrap();

		let object = room.get_object(&object_id).unwrap();
		assert_eq!(*object.get_long(&10).unwrap(), 100);
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if c==command));
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
		assert_eq!(*object.get_long(&10).unwrap(), 200);

		let result = SetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 200,
		};

		room.test_out_commands.pop_back();
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if c==result));
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
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(user_id), access_groups);
		object.created = true;
		let object_id = object.id.clone();
		(room, user_id, object_id)
	}
}
