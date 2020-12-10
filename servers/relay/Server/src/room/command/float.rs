use cheetah_relay_common::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::ServerCommandExecutor;
use crate::room::object::GameObject;
use crate::room::Room;

impl ServerCommandExecutor for IncrementFloat64C2SCommand {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object_mut(&self.object_id) {
			let value = if let Some(value) = object.floats.get_mut(&self.field_id) {
				*value += self.increment;
				*value
			} else {
				object.floats.insert(self.field_id, self.increment);
				self.increment
			};

			let access_groups = object.access_groups.clone();
			room.send_to_group(
				access_groups,
				S2CCommand::SetFloat64(SetFloat64Command {
					object_id: self.object_id,
					field_id: self.field_id,
					value,
				}),
			);
		}
	}
}

impl ServerCommandExecutor for SetFloat64Command {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object_mut(&self.object_id) {
			object.floats.insert(self.field_id, self.value);
			let access_groups = object.access_groups;
			room.send_to_group(access_groups, S2CCommand::SetFloat64(self));
		}
	}
}

impl GameObject {
	pub fn floats_to_commands(&self, commands: &mut Vec<S2CCommand>) {
		self.floats.iter().for_each(|(k, v)| {
			commands.push(S2CCommand::SetFloat64(SetFloat64Command {
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
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::RoomTemplate;
	use crate::room::Room;

	#[test]
	fn should_set_float_command() {
		let mut room = Room::new(RoomTemplate::default(), Default::default());
		let object_id = room.create_object(&0).id.clone();
		room.out_commands.clear();
		let command = SetFloat64Command {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100.100,
		};
		command.clone().execute(&mut room, &12);

		let object = room.get_object_mut(&object_id).unwrap();
		assert_eq!(*object.floats.get(&10).unwrap() as u64, 100);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetFloat64(c))) if c==command));
	}

	#[test]
	fn should_increment_float_command() {
		let mut room = Room::new(RoomTemplate::default(), Default::default());
		let object_id = room.create_object(&0).id.clone();
		room.out_commands.clear();
		let command = IncrementFloat64C2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: 100.100,
		};
		command.clone().execute(&mut room, &12);
		command.clone().execute(&mut room, &12);

		let object = room.get_object_mut(&object_id).unwrap();
		assert_eq!(*object.floats.get(&10).unwrap() as u64, 200);

		let result = SetFloat64Command {
			object_id: object_id.clone(),
			field_id: 10,
			value: 200.200,
		};
		room.out_commands.pop_back();
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetFloat64(c))) if c==result));
	}

	#[test]
	fn should_not_panic_when_set_float_command_not_panic_for_missing_object() {
		let mut room = Room::new(RoomTemplate::default(), Default::default());
		let command = SetFloat64Command {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 10,
			value: 100.100,
		};
		command.execute(&mut room, &12);
	}

	#[test]
	fn should_not_panic_when_increment_float_command_not_panic_for_missing_object() {
		let mut room = Room::new(RoomTemplate::default(), Default::default());
		let command = IncrementFloat64C2SCommand {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 10,
			increment: 100.100,
		};
		command.execute(&mut room, &12);
	}
}
