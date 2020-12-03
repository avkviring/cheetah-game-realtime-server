use cheetah_relay_common::commands::command::long::{IncrementLongC2SCommand, SetLongCommand};
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::ServerCommandExecutor;
use crate::room::Room;

impl ServerCommandExecutor for IncrementLongC2SCommand {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			let value = if let Some(value) = object.fields.longs.get_mut(&self.field_id) {
				match (*value).checked_add(self.increment) {
					None => {
						log::error!("[IncrementLongC2SCommand] overflow, current({:?}) increment({:?})", value, self.increment);
					}
					Some(result) => {
						*value = result;
					}
				}
				*value
			} else {
				match object.fields.longs.insert(self.field_id, self.increment) {
					Ok(_) => {}
					Err(_) => {
						log::error!("[IncrementLong] overflow element count in object({:?})", object.id);
						return;
					}
				}
				self.increment
			};

			let access_groups = object.access_groups.clone();
			room.send_to_group(
				false,
				access_groups,
				S2CCommand::SetLong(SetLongCommand {
					object_id: self.object_id,
					field_id: self.field_id,
					value,
				}),
			);
		}
	}
}

impl ServerCommandExecutor for SetLongCommand {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			match object.fields.longs.insert(self.field_id, self.value) {
				Ok(_) => {
					let access_groups = object.access_groups.clone();
					room.send_to_group(false, access_groups, S2CCommand::SetLong(self));
				}
				Err(_) => {
					log::error!("[SetLongCommand] overflow element count in object({:?})", object.id);
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::long::{IncrementLongC2SCommand, SetLongCommand};
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::RoomTemplate;
	use crate::room::Room;

	#[test]
	fn should_set_long_command() {
		let mut room = Room::new(RoomTemplate::default(), Default::default());
		let object_id = room.create_object(&0).id.clone();
		room.out_commands.clear();
		let command = SetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100,
		};
		command.clone().execute(&mut room, &12);

		let object = room.get_object(&object_id).unwrap();
		assert_eq!(*object.fields.longs.get(&10).unwrap(), 100);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if c==command));
	}

	#[test]
	fn should_increment_long_command() {
		let mut room = Room::new(RoomTemplate::default(), Default::default());
		let object_id = room.create_object(&0).id.clone();
		room.out_commands.clear();
		let command = IncrementLongC2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: 100,
		};
		command.clone().execute(&mut room, &12);
		command.clone().execute(&mut room, &12);

		let object = room.get_object(&object_id).unwrap();
		assert_eq!(*object.fields.longs.get(&10).unwrap(), 200);

		let result = SetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 200,
		};
		room.out_commands.pop_back();
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if c==result));
	}

	#[test]
	fn should_not_panic_when_set_long_command_not_panic_for_missing_object() {
		let mut room = Room::new(RoomTemplate::default(), Default::default());
		let command = SetLongCommand {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 10,
			value: 100,
		};
		command.execute(&mut room, &12);
	}

	#[test]
	fn should_not_panic_when_increment_float_command_not_panic_for_missing_object() {
		let mut room = Room::new(RoomTemplate::default(), Default::default());
		let command = IncrementLongC2SCommand {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 10,
			increment: 100,
		};
		command.execute(&mut room, &12);
	}

	#[test]
	fn should_not_panic_if_overflow() {
		let mut room = Room::new(RoomTemplate::default(), Default::default());
		let object_id = room.create_object(&0).id.clone();
		room.out_commands.clear();
		let command = IncrementLongC2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: i64::max_value(),
		};
		command.clone().execute(&mut room, &12);
		command.execute(&mut room, &12);
	}
}
