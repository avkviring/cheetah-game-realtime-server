use cheetah_matches_relay_common::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
use cheetah_matches_relay_common::commands::command::S2CCommand;
use cheetah_matches_relay_common::room::UserId;

use crate::room::command::ServerCommandExecutor;
use crate::room::object::{FieldIdAndType, GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::Permission;
use crate::room::types::FieldType;
use crate::room::Room;

impl ServerCommandExecutor for IncrementFloat64C2SCommand {
	fn execute(self, room: &mut Room, user_id: UserId) {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();

		let action = |object: &mut GameObject| {
			let value = if let Some(value) = object.floats.get_mut(&field_id) {
				*value += self.increment;
				*value
			} else {
				object.floats.insert(field_id, self.increment);
				self.increment
			};
			Option::Some(S2CCommand::SetFloat(SetFloat64Command {
				object_id: self.object_id.clone(),
				field_id,
				value,
			}))
		};

		room.change_data_and_send(
			&object_id,
			&field_id,
			FieldType::Double,
			user_id,
			Permission::Rw,
			Option::None,
			action,
		);
	}
}

impl ServerCommandExecutor for SetFloat64Command {
	fn execute(self, room: &mut Room, user_id: UserId) {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();

		let action = |object: &mut GameObject| {
			object.floats.insert(self.field_id, self.value);
			Option::Some(S2CCommand::SetFloat(self))
		};
		room.change_data_and_send(
			&object_id,
			&field_id,
			FieldType::Double,
			user_id,
			Permission::Rw,
			Option::None,
			action,
		);
	}
}

impl GameObject {
	pub fn floats_to_commands(&self, commands: &mut Vec<S2CommandWithFieldInfo>) {
		self.floats.iter().for_each(|(field_id, v)| {
			commands.push(S2CommandWithFieldInfo {
				field: Option::Some(FieldIdAndType {
					field_id: field_id.clone(),
					field_type: FieldType::Double,
				}),
				command: S2CCommand::SetFloat(SetFloat64Command {
					object_id: self.id.clone(),
					field_id: field_id.clone(),
					value: *v,
				}),
			});
		})
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
	use cheetah_matches_relay_common::commands::command::S2CCommand;
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;
	use cheetah_matches_relay_common::room::UserId;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::{RoomTemplate, UserTemplate};
	use crate::room::Room;

	#[test]
	fn should_set_float_command() {
		let (mut room, user, access_groups) = setup();
		let object = room.create_object(user, access_groups);
		let object_id = object.id.clone();
		object.created = true;
		room.out_commands.clear();
		let command = SetFloat64Command {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100.100,
		};
		command.clone().execute(&mut room, user);

		let object = room.get_object_mut(&object_id).unwrap();
		assert_eq!(*object.floats.get(&10).unwrap() as u64, 100);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetFloat(c))) if c==command));
	}

	#[test]
	fn should_increment_float_command() {
		let (mut room, user, access_groups) = setup();

		let object = room.create_object(user, access_groups);
		object.created = true;
		let object_id = object.id.clone();
		room.out_commands.clear();
		let command = IncrementFloat64C2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: 100.100,
		};
		command.clone().execute(&mut room, user);
		command.clone().execute(&mut room, user);

		let object = room.get_object_mut(&object_id).unwrap();
		assert_eq!(*object.floats.get(&10).unwrap() as u64, 200);

		let result = SetFloat64Command {
			object_id: object_id.clone(),
			field_id: 10,
			value: 200.200,
		};
		room.out_commands.pop_back();
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetFloat(c))) if c==result));
	}

	#[test]
	fn should_not_panic_when_increment_float_command_not_panic_for_missing_object() {
		let (mut room, user, _) = setup();

		let command = IncrementFloat64C2SCommand {
			object_id: GameObjectId::new(10, GameObjectOwner::Room),
			field_id: 10,
			increment: 100.100,
		};
		command.execute(&mut room, user);
	}

	fn setup() -> (Room, UserId, AccessGroups) {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::from_template(template);
		let user_id = room.register_user(UserTemplate::stub(access_groups));
		(room, user_id, access_groups)
	}
}
