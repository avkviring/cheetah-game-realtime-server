use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fnv::FnvBuildHasher;

use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand};
use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::command::ServerCommandExecutor;
use crate::room::object::{CreateCommandsCollector, Field, GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::Permission;
use crate::room::Room;

impl ServerCommandExecutor for IncrementLongC2SCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) {
		let action = |object: &mut GameObject| {
			let value = if let Some(value) = object.longs.get_mut(&self.field_id) {
				match (*value).checked_add(self.increment) {
					None => {
						log::error!(
							"[IncrementLongC2SCommand] overflow, current({:?}) increment({:?})",
							value,
							self.increment
						);
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
			Option::Some(S2CCommand::SetLong(SetLongCommand {
				object_id: self.object_id.clone(),
				field_id: self.field_id,
				value,
			}))
		};
		room.do_action_and_send_commands(
			&self.object_id,
			Field {
				id: self.field_id,
				field_type: FieldType::Long,
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		);
	}
}

impl ServerCommandExecutor for SetLongCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();

		let action = |object: &mut GameObject| {
			object.longs.insert(self.field_id, self.value);
			Option::Some(S2CCommand::SetLong(self.clone()))
		};

		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id,
				field_type: FieldType::Long,
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		);
	}
}

impl ServerCommandExecutor for CompareAndSetLongCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) {
		let object_id = self.object_id.clone();
		let field_id = self.field_id;
		let reset = self.reset;

		let is_set = Rc::new(RefCell::new(false));
		let is_set_cloned = is_set.clone();

		let action = |object: &mut GameObject| {
			let allow = match object.longs.get(&self.field_id) {
				None => true,
				Some(value) => *value == self.current,
			};
			if allow {
				object.longs.insert(self.field_id, self.new);
				object.compare_and_set_owners.insert(self.field_id, user_id);
				*is_set_cloned.borrow_mut() = true;
				Option::Some(S2CCommand::SetLong(SetLongCommand {
					object_id: self.object_id.clone(),
					field_id: self.field_id,
					value: self.new,
				}))
			} else {
				Option::None
			}
		};

		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id,
				field_type: FieldType::Long,
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		);

		if *(is_set.borrow()) {
			room.get_member_mut(user_id)
				.unwrap()
				.compare_and_sets_cleaners
				.insert((object_id, field_id), reset);
		}
	}
}

pub fn reset_all_compare_and_set(
	room: &mut Room,
	user_id: RoomMemberId,
	compare_and_sets_cleaners: HashMap<(GameObjectId, FieldId), i64, FnvBuildHasher>,
) {
	for ((object_id, field), reset) in compare_and_sets_cleaners {
		match room.get_object_mut(&object_id) {
			None => {
				// нормальная ситуация для пользовательских объектов
			}
			Some(object) => {
				if let Some(owner) = object.compare_and_set_owners.get(&field) {
					if *owner == user_id {
						object.longs.insert(field, reset);
						let command = [S2CommandWithFieldInfo {
							field: Some(Field {
								id: field,
								field_type: FieldType::Long,
							}),
							command: S2CCommand::SetLong(SetLongCommand {
								object_id,
								field_id: field,
								value: reset,
							}),
						}];
						let groups = object.access_groups;
						let template = object.template;
						room.send_to_members(groups, template, &command, |_| true)
					}
				}
			}
		}
	}
}

impl GameObject {
	pub fn longs_to_commands(&self, commands: &mut CreateCommandsCollector) -> Result<(), S2CommandWithFieldInfo> {
		for (field_id, v) in &self.longs {
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
	use cheetah_matches_relay_common::commands::types::long::{
		CompareAndSetLongCommand, IncrementLongC2SCommand, SetLongCommand,
	};
	use cheetah_matches_relay_common::commands::FieldType;
	use cheetah_matches_relay_common::constants::FieldId;
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::RoomMemberId;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::object::Field;
	use crate::room::template::config::{
		GameObjectTemplatePermission, GroupsPermissionRule, Permission, PermissionField, RoomTemplate, UserTemplate,
	};
	use crate::room::Room;

	#[test]
	fn should_set_long_command() {
		let (mut room, user, object_id) = setup();

		room.out_commands.clear();
		let command = SetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100,
		};
		command.clone().execute(&mut room, user);

		let object = room.get_object_mut(&object_id).unwrap();
		assert_eq!(*object.longs.get(&10).unwrap(), 100);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if c==command));
	}

	#[test]
	fn should_increment_long_command() {
		let (mut room, user, object_id) = setup();

		room.out_commands.clear();
		let command = IncrementLongC2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: 100,
		};
		command.clone().execute(&mut room, user);
		command.execute(&mut room, user);

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
	fn should_not_panic_if_overflow() {
		let (mut room, user, object_id) = setup();
		room.out_commands.clear();
		let command = IncrementLongC2SCommand {
			object_id,
			field_id: 10,
			increment: i64::MAX,
		};
		command.clone().execute(&mut room, user);
		command.execute(&mut room, user);
	}

	///
	/// Проверяем что при выполнении нескольких команд соблюдаются гарантии CompareAndSet
	///
	#[test]
	fn test_compare_and_set() {
		let (mut room, user1_id, _, object_id, field_id) = setup_for_compare_and_set();
		let command1 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id,
			current: 0,
			new: 100,
			reset: 0,
		};
		command1.clone().execute(&mut room, user1_id);
		assert_eq!(
			*room
				.get_object_mut(&object_id)
				.unwrap()
				.longs
				.get(&command1.field_id)
				.unwrap(),
			command1.new
		);

		let command2 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id: command1.field_id,
			current: 0,
			new: 200,
			reset: 0,
		};
		command2.execute(&mut room, user1_id);
		assert_eq!(
			*room
				.get_object_mut(&object_id)
				.unwrap()
				.longs
				.get(&command1.field_id)
				.unwrap(),
			command1.new
		);

		let command3 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id: command1.field_id,
			current: command1.new,
			new: 300,
			reset: 0,
		};
		command3.clone().execute(&mut room, user1_id);
		assert_eq!(
			*room
				.get_object_mut(&object_id)
				.unwrap()
				.longs
				.get(&command1.field_id)
				.unwrap(),
			command3.new
		);
	}

	///
	/// Проверяем что команда отсылает изменения другим клиентам
	#[test]
	fn test_compare_and_set_1() {
		let (mut room, user1_id, _, object_id, field_id) = setup_for_compare_and_set();
		let command = CompareAndSetLongCommand {
			object_id,
			field_id,
			current: 0,
			new: 100,
			reset: 555,
		};

		room.out_commands.clear();
		command.clone().execute(&mut room, user1_id);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if c.value==command.new));
	}

	///
	/// Проверяем что при выходе пользователя будет установлено заданное значение
	///
	#[test]
	fn test_compare_and_set_2() {
		let (mut room, user1_id, _, object_id, field_id) = setup_for_compare_and_set();
		let command = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id,
			current: 0,
			new: 100,
			reset: 555,
		};
		command.clone().execute(&mut room, user1_id);
		assert_eq!(
			*room.get_object_mut(&object_id).unwrap().longs.get(&command.field_id).unwrap(),
			command.new
		);

		room.disconnect_user(user1_id);
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
		let (mut room, user1_id, user2_id, object_id, field_id) = setup_for_compare_and_set();
		let command_1 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id,
			current: 0,
			new: 100,
			reset: 555,
		};
		let command_2 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id,
			current: 100,
			new: 200,
			reset: 1555,
		};
		command_1.clone().execute(&mut room, user1_id);
		command_2.clone().execute(&mut room, user2_id);

		room.disconnect_user(user1_id);
		assert_eq!(
			*room
				.get_object_mut(&object_id)
				.unwrap()
				.longs
				.get(&command_1.field_id)
				.unwrap(),
			command_2.new
		);
	}

	fn setup_for_compare_and_set() -> (Room, RoomMemberId, RoomMemberId, GameObjectId, FieldId) {
		let access_group = AccessGroups(55);
		let mut template = RoomTemplate::default();
		let user_template_1 = UserTemplate {
			private_key: Default::default(),
			groups: access_group,
			objects: Default::default(),
		};
		let user_template_2 = UserTemplate {
			private_key: Default::default(),
			groups: access_group,
			objects: Default::default(),
		};

		let user_template_3 = UserTemplate {
			private_key: Default::default(),
			groups: access_group,
			objects: Default::default(),
		};

		let object_template = 10;
		let object_field = 50;
		template.permissions.templates.push(GameObjectTemplatePermission {
			template: object_template,
			rules: vec![],
			fields: vec![PermissionField {
				field: Field {
					id: object_field,
					field_type: FieldType::Long,
				},
				rules: vec![GroupsPermissionRule {
					groups: access_group,
					permission: Permission::Rw,
				}],
			}],
		});
		let mut room = Room::from_template(template);
		let user1_id = room.register_user(user_template_1);
		let user2_id = room.register_user(user_template_2);
		let user3_id = room.register_user(user_template_3);
		let object = room.create_object(user3_id, access_group);
		object.created = true;
		object.template = object_template;

		let object_id = object.id.clone();
		(room, user1_id, user2_id, object_id, object_field)
	}

	fn setup() -> (Room, RoomMemberId, GameObjectId) {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::from_template(template);
		let user_id = room.register_user(UserTemplate::stub(access_groups));
		let object = room.create_object(user_id, access_groups);
		object.created = true;
		let object_id = object.id.clone();
		(room, user_id, object_id)
	}
}
