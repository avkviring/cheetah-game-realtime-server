use std::collections::HashMap;

use fnv::FnvBuildHasher;

use cheetah_relay_common::commands::command::long::{CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand};
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::constants::FieldIdType;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::ServerCommandExecutor;
use crate::room::object::GameObject;
use crate::room::Room;

impl ServerCommandExecutor for IncrementLongC2SCommand {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object_mut(&self.object_id) {
			let value = if let Some(value) = object.longs.get_mut(&self.field_id) {
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
				object.longs.insert(self.field_id, self.increment);
				self.increment
			};

			let access_groups = object.access_groups.clone();
			room.send_to_group(
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
		if let Some(object) = room.get_object_mut(&self.object_id) {
			object.longs.insert(self.field_id, self.value);
			let access_groups = object.access_groups.clone();
			room.send_to_group(access_groups, S2CCommand::SetLong(self));
		}
	}
}

impl ServerCommandExecutor for CompareAndSetLongCommand {
	fn execute(self, room: &mut Room, user_public_key: &UserPublicKey) {
		if let Some(object) = room.get_object_mut(&self.object_id) {
			let allow = match object.longs.get(&self.field_id) {
				None => true,
				Some(value) => *value == self.current,
			};
			if allow {
				object.longs.insert(self.field_id, self.new);
				object.compare_and_set_owners.insert(self.field_id, user_public_key.clone());
				let object_id = object.id.clone();

				let access_groups = object.access_groups.clone();
				room.send_to_group(
					access_groups,
					S2CCommand::SetLong(SetLongCommand {
						object_id: self.object_id,
						field_id: self.field_id,
						value: self.new,
					}),
				);

				room.get_user_mut(user_public_key)
					.unwrap()
					.compare_and_sets_cleaners
					.insert((object_id, self.field_id), self.reset);
			}
		}
	}
}

pub fn reset_all_compare_and_set(
	room: &mut Room,
	user_public_key: UserPublicKey,
	compare_and_sets_cleaners: HashMap<(GameObjectId, FieldIdType), i64, FnvBuildHasher>,
) {
	for ((object_id, field), reset) in compare_and_sets_cleaners {
		match room.get_object_mut(&object_id) {
			None => {}
			Some(object) => {
				if let Some(owner) = object.compare_and_set_owners.get(&field) {
					if *owner == user_public_key {
						object.longs.insert(field, reset);
					}
				}
			}
		}
	}
}

impl GameObject {
	pub fn longs_to_commands(&self, commands: &mut Vec<S2CCommand>) {
		self.longs.iter().for_each(|(k, v)| {
			commands.push(S2CCommand::SetLong(SetLongCommand {
				object_id: self.id.clone(),
				field_id: k.clone(),
				value: *v,
			}));
		})
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::long::{CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand};
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::{RoomTemplate, UserTemplate};
	use crate::room::Room;

	#[test]
	fn should_set_long_command() {
		let mut room = Room::default();
		let object_id = room.create_object(&0).id.clone();
		room.out_commands.clear();
		let command = SetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100,
		};
		command.clone().execute(&mut room, &12);

		let object = room.get_object_mut(&object_id).unwrap();
		assert_eq!(*object.longs.get(&10).unwrap(), 100);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if c==command));
	}

	#[test]
	fn should_increment_long_command() {
		let mut room = Room::default();
		let object_id = room.create_object(&0).id.clone();
		room.out_commands.clear();
		let command = IncrementLongC2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: 100,
		};
		command.clone().execute(&mut room, &12);
		command.clone().execute(&mut room, &12);

		let object = room.get_object_mut(&object_id).unwrap();
		assert_eq!(*object.longs.get(&10).unwrap(), 200);

		let result = SetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 200,
		};
		room.out_commands.pop_back();
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if c==result));
	}

	#[test]
	fn should_not_panic_when_set_long_command_for_missing_object() {
		let mut room = Room::default();
		let command = SetLongCommand {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 10,
			value: 100,
		};
		command.execute(&mut room, &12);
	}

	#[test]
	fn should_not_panic_when_increment_command_for_missing_object() {
		let mut room = Room::default();
		let command = IncrementLongC2SCommand {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 10,
			increment: 100,
		};
		command.execute(&mut room, &12);
	}

	#[test]
	fn should_not_panic_if_overflow() {
		let mut room = Room::default();
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

	///
	/// Проверяем что при выполнении нескольких команд соблюдаются гарантии CompareAndSet
	///
	#[test]
	fn test_compare_and_set() {
		let (mut room, user_template, _, object_id) = setup_for_compare_and_set();
		let command1 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			current: 0,
			new: 100,
			reset: 0,
		};
		command1.clone().execute(&mut room, &user_template.public_key);
		assert_eq!(
			*room.get_object_mut(&object_id).unwrap().longs.get(&command1.field_id).unwrap(),
			command1.new
		);

		let command2 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id: command1.field_id,
			current: 0,
			new: 200,
			reset: 0,
		};
		command2.clone().execute(&mut room, &user_template.public_key);
		assert_eq!(
			*room.get_object_mut(&object_id).unwrap().longs.get(&command1.field_id).unwrap(),
			command1.new
		);

		let command3 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id: command1.field_id,
			current: command1.new,
			new: 300,
			reset: 0,
		};
		command3.clone().execute(&mut room, &user_template.public_key);
		assert_eq!(
			*room.get_object_mut(&object_id).unwrap().longs.get(&command1.field_id).unwrap(),
			command3.new
		);
	}
	///
	/// Проверяем что команда отсылает изменения другим клиентам
	#[test]
	fn test_compare_and_set_1() {
		let (mut room, user_template, _, object_id) = setup_for_compare_and_set();
		let command = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			current: 0,
			new: 100,
			reset: 555,
		};
		room.out_commands.clear();
		command.clone().execute(&mut room, &user_template.public_key);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if c.value==command.new));
	}

	///
	/// Проверяем что при выходе пользователя будет установлено заданное значение
	///
	#[test]
	fn test_compare_and_set_2() {
		let (mut room, user_template, _, object_id) = setup_for_compare_and_set();
		let command = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			current: 0,
			new: 100,
			reset: 555,
		};
		command.clone().execute(&mut room, &user_template.public_key);
		assert_eq!(
			*room.get_object_mut(&object_id).unwrap().longs.get(&command.field_id).unwrap(),
			command.new
		);

		room.disconnect_user(&user_template.public_key);
		assert_eq!(
			*room.get_object_mut(&object_id).unwrap().longs.get(&command.field_id).unwrap(),
			command.reset
		);
	}

	///
	/// Проверяем,что если два пользователя друг за другом поменяли значения,
	/// то при разрыве соединения первого пользователя данные не будут заменены
	///
	#[test]
	fn test_compare_and_set_3() {
		let (mut room, user_template_1, user_template_2, object_id) = setup_for_compare_and_set();
		let command_1 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			current: 0,
			new: 100,
			reset: 555,
		};
		let command_2 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			current: 100,
			new: 200,
			reset: 1555,
		};
		command_1.clone().execute(&mut room, &user_template_1.public_key);
		command_2.clone().execute(&mut room, &user_template_2.public_key);

		room.disconnect_user(&user_template_1.public_key);
		assert_eq!(
			*room.get_object_mut(&object_id).unwrap().longs.get(&command_1.field_id).unwrap(),
			command_2.new
		);
	}

	fn setup_for_compare_and_set() -> (Room, UserTemplate, UserTemplate, GameObjectId) {
		let mut template = RoomTemplate::default();
		let user_template_1 = UserTemplate {
			public_key: 55,
			private_key: Default::default(),
			access_groups: Default::default(),
			objects: Default::default(),
			unmapping: Default::default(),
		};
		let user_template_2 = UserTemplate {
			public_key: 155,
			private_key: Default::default(),
			access_groups: Default::default(),
			objects: Default::default(),
			unmapping: Default::default(),
		};
		template.users.push(user_template_1.clone());
		template.users.push(user_template_2.clone());
		let mut room = Room::new_with_template(template);
		let object_id = room.create_object(&1).id.clone();
		(room, user_template_1, user_template_2, object_id)
	}
}
