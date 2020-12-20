use cheetah_relay_common::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::ServerCommandExecutor;
use crate::room::object::GameObject;
use crate::room::template::config::Permission;
use crate::room::types::FieldType;
use crate::room::Room;

impl ServerCommandExecutor for IncrementFloat64C2SCommand {
	fn execute(self, room: &mut Room, user_public_key: &UserPublicKey) {
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

		room.do_action(&object_id, &field_id, FieldType::Float, user_public_key, Permission::Rw, action);
	}
}

impl ServerCommandExecutor for SetFloat64Command {
	fn execute(self, room: &mut Room, user_public_key: &UserPublicKey) {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();

		let action = |object: &mut GameObject| {
			object.floats.insert(self.field_id, self.value);
			Option::Some(S2CCommand::SetFloat(self))
		};
		room.do_action(&object_id, &field_id, FieldType::Float, user_public_key, Permission::Rw, action);
	}
}

impl GameObject {
	pub fn floats_to_commands(&self, commands: &mut Vec<S2CCommand>) {
		self.floats.iter().for_each(|(k, v)| {
			commands.push(S2CCommand::SetFloat(SetFloat64Command {
				object_id: self.id.clone(),
				field_id: k.clone(),
				value: *v,
			}));
		})
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::RoomTemplate;
	use crate::room::Room;

	#[test]
	fn should_set_float_command() {
		let mut template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let user = template.create_user(1, access_groups);
		let mut room = Room::from_template(template);

		let object_id = room.create_object(&user, access_groups).id.clone();
		room.out_commands.clear();
		let command = SetFloat64Command {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100.100,
		};
		command.clone().execute(&mut room, &user);

		let object = room.get_object_mut(&object_id).unwrap();
		assert_eq!(*object.floats.get(&10).unwrap() as u64, 100);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetFloat(c))) if c==command));
	}

	#[test]
	fn should_increment_float_command() {
		let mut template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let user = template.create_user(1, access_groups);
		let mut room = Room::from_template(template);

		let object_id = room.create_object(&user, access_groups).id.clone();
		room.out_commands.clear();
		let command = IncrementFloat64C2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: 100.100,
		};
		command.clone().execute(&mut room, &user);
		command.clone().execute(&mut room, &user);

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
		let mut template = RoomTemplate::default();
		let user = template.create_user(1, AccessGroups(10));
		let mut room = Room::from_template(template);

		let command = IncrementFloat64C2SCommand {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 10,
			increment: 100.100,
		};
		command.execute(&mut room, &user);
	}
}
