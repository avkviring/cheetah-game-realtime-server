use cheetah_relay_common::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::ServerCommandExecutor;
use crate::room::Room;

impl ServerCommandExecutor for IncrementFloat64C2SCommand {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			let value = if let Some(value) = object.fields.floats.get_mut(&self.field_id) {
				*value += self.increment;
				*value
			} else {
				match object.fields.floats.insert(self.field_id, self.increment) {
					Ok(_) => {}
					Err(_) => {
						log::error!("[IncrementFloatCommand] overflow element count in object({:?})", object.id);
						return;
					}
				}
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
		if let Some(object) = room.get_object(&self.object_id) {
			match object.fields.floats.insert(self.field_id, self.value) {
				Ok(_) => {}
				Err(_) => {
					log::error!("[SetFloatCommand] overflow element count in object({:?})", object.id);
					return;
				}
			}
			let access_groups = object.access_groups;
			room.send_to_group(access_groups, S2CCommand::SetFloat64(self));
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::float::{IncrementFloat64C2SCommand, SetFloat64Command};
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::Room;

	#[test]
	fn should_set_float_command() {
		let mut room = Room::new(0, false);
		let object_id = room.create_object(&0).id.clone();
		let command = SetFloat64Command {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100.100,
		};
		command.clone().execute(&mut room, &12);

		let object = room.get_object(&object_id).unwrap();
		assert_eq!(*object.fields.floats.get(&10).unwrap() as u64, 100);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetFloat64(c))) if c==command));
	}

	#[test]
	fn should_increment_float_command() {
		let mut room = Room::new(0, false);
		let object_id = room.create_object(&0).id.clone();
		let command = IncrementFloat64C2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: 100.100,
		};
		command.clone().execute(&mut room, &12);
		command.clone().execute(&mut room, &12);

		let object = room.get_object(&object_id).unwrap();
		assert_eq!(*object.fields.floats.get(&10).unwrap() as u64, 200);

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
		let mut room = Room::new(0, false);
		let command = SetFloat64Command {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 10,
			value: 100.100,
		};
		command.execute(&mut room, &12);
	}

	#[test]
	fn should_not_panic_when_increment_float_command_not_panic_for_missing_object() {
		let mut room = Room::new(0, false);
		let command = IncrementFloat64C2SCommand {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 10,
			increment: 100.100,
		};
		command.execute(&mut room, &12);
	}
}
