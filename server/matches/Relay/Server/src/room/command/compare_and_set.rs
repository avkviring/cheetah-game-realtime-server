use std::cell::RefCell;
use std::rc::Rc;

use cheetah_matches_relay_common::commands::binary_value::BinaryValue;
use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::structure::SetStructureCommand;
use cheetah_matches_relay_common::commands::types::{
	long::{CompareAndSetLongCommand, SetLongCommand},
	structure::{CompareAndSetStructureCommand},
};
use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::{Field, GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::Permission;
use crate::room::Room;

#[derive(Debug)]
pub enum ResetValue {
	Long(i64),
	Structure(BinaryValue),
}

impl ServerCommandExecutor for CompareAndSetLongCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let object_id = self.object_id.clone();
		let field_id = self.field_id;
		let is_field_changed = Rc::new(RefCell::new(false));
		let action = |object: &mut GameObject| {
			let allow = match object.get_long(&self.field_id) {
				None => true,
				Some(value) => *value == self.current,
			};
			if allow {
				*is_field_changed.borrow_mut() = true;
				object.set_long(self.field_id, self.new)?;
				if self.reset.is_some() {
					object.set_compare_and_set_owner(self.field_id, user_id)?;
				}
				Ok(Some(S2CCommand::SetLong(SetLongCommand {
					object_id: self.object_id.clone(),
					field_id: self.field_id,
					value: self.new,
				})))
			} else {
				Ok(None)
			}
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
		)?;

		if is_field_changed.take() {
			match self.reset {
				None => {
					room.get_member_mut(&user_id)?
						.compare_and_set_cleaners
						.remove(&(object_id, field_id));
				}
				Some(reset_value) => {
					room.get_member_mut(&user_id)?
						.compare_and_set_cleaners
						.insert((object_id, field_id), ResetValue::Long(reset_value))
						.map_err(|_| ServerCommandError::Error("Overflow compare and sets cleaners".to_string()))?;
				}
			}
		}

		Ok(())
	}
}

impl ServerCommandExecutor for CompareAndSetStructureCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let object_id = self.object_id.clone();
		let field_id = self.field_id;
		let is_field_changed = Rc::new(RefCell::new(false));
		let action = |object: &mut GameObject| {
			let allow = match object.get_structure(&self.field_id) {
				None => true,
				Some(value) => value == self.current.as_slice(),
			};
			if allow {
				*is_field_changed.borrow_mut() = true;
				object.set_structure(self.field_id, self.new.as_slice())?;
				if self.reset.is_some() {
					object.set_compare_and_set_owner(self.field_id, user_id)?;
				}
				Ok(Some(S2CCommand::SetStructure(SetStructureCommand {
					object_id: self.object_id.clone(),
					field_id: self.field_id,
					structure: self.new.clone(),
				})))
			} else {
				Ok(None)
			}
		};

		room.send_command_from_action(
			&object_id,
			Field {
				id: field_id,
				field_type: FieldType::Structure,
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		)?;

		if is_field_changed.take() {
			match self.reset.clone() {
				None => {
					room.get_member_mut(&user_id)?
						.compare_and_set_cleaners
						.remove(&(object_id, field_id));
				}
				Some(reset_value) => {
					room.get_member_mut(&user_id)?
						.compare_and_set_cleaners
						.insert((object_id, field_id), ResetValue::Structure(reset_value))
						.map_err(|_| ServerCommandError::Error("CompareAndSetCleaners overflow".to_string()))?;
				}
			}
		}

		Ok(())
	}
}

pub fn reset_all_compare_and_set(
	room: &mut Room,
	user_id: RoomMemberId,
	compare_and_set_cleaners: &heapless::FnvIndexMap<(GameObjectId, FieldId), ResetValue, 256>,
) -> Result<(), ServerCommandError> {
	for ((object_id, field), reset) in compare_and_set_cleaners {
		match room.get_object(object_id) {
			Err(_) => {
				// нормальная ситуация для пользовательских объектов
			}
			Ok(object) => {
				if let Some(owner) = object.get_compare_and_set_owner(field) {
					if *owner == user_id {
						let command: [S2CommandWithFieldInfo; 1];
						match reset {
							ResetValue::Long(value) => {
								object.set_long(*field, *value)?;
								command = [S2CommandWithFieldInfo {
									field: Some(Field {
										id: *field,
										field_type: FieldType::Long,
									}),
									command: S2CCommand::SetLong(SetLongCommand {
										object_id: object_id.clone(),
										field_id: *field,
										value: *value,
									}),
								}];
							},
							ResetValue::Structure(structure) => {
								object.set_structure(*field, structure.as_slice())?;
								command = [S2CommandWithFieldInfo {
									field: Some(Field {
										id: *field,
										field_type: FieldType::Structure,
									}),
									command: S2CCommand::SetStructure(SetStructureCommand {
										object_id: object_id.clone(),
										field_id: *field,
										structure: structure.to_owned(),
									})
								}];
							},
						}
						let groups = object.access_groups;
						let template = object.template_id;
						room.send_to_members(
							groups, template,
							&command, |_| true
						)?
					}
				}
			}
		}
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::s2c::S2CCommand;
	use cheetah_matches_relay_common::commands::types::long::CompareAndSetLongCommand;
	use cheetah_matches_relay_common::commands::FieldType;
	use cheetah_matches_relay_common::commands::types::structure::CompareAndSetStructureCommand;
	use cheetah_matches_relay_common::constants::FieldId;
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;
	use cheetah_matches_relay_common::room::RoomMemberId;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::object::Field;
	use crate::room::template::config::{
		GameObjectTemplatePermission, GroupsPermissionRule, MemberTemplate, Permission, PermissionField, RoomTemplate,
	};
	use crate::room::Room;

	///
	/// Проверяем что при выполнении нескольких команд соблюдаются гарантии CompareAndSet
	///
	#[test]
	fn should_compare_and_set_long() {
		let (mut room, user1_id, _, object_id, field_id) = setup();
		let command1 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id,
			current: 0,
			new: 100,
			reset: None,
		};
		command1.execute(&mut room, user1_id).unwrap();
		assert_eq!(
			*room.get_object(&object_id).unwrap().get_long(&command1.field_id).unwrap(),
			command1.new
		);

		let command2 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id: command1.field_id,
			current: 0,
			new: 200,
			reset: None,
		};
		command2.execute(&mut room, user1_id).unwrap();
		assert_eq!(
			*room.get_object(&object_id).unwrap().get_long(&command1.field_id).unwrap(),
			command1.new
		);

		let command3 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id: command1.field_id,
			current: command1.new,
			new: 300,
			reset: None,
		};
		command3.execute(&mut room, user1_id).unwrap();
		assert_eq!(
			*room.get_object(&object_id).unwrap().get_long(&command1.field_id).unwrap(),
			command3.new
		);
	}

	#[test]
	fn should_compare_and_set_structure() {
		let (mut room, user1_id, _, object_id, field_id) = setup();
		let command1 = CompareAndSetStructureCommand {
			object_id: object_id.clone(),
			field_id,
			current: vec![0, 1].as_slice().into(),
			new: vec![1, 0, 0].as_slice().into(),
			reset: None,
		};
		command1.execute(&mut room, user1_id).unwrap();
		assert_eq!(
			*room.get_object(&object_id).unwrap().get_structure(&command1.field_id).unwrap(),
			command1.new.as_slice()
		);

		let command2 = CompareAndSetStructureCommand {
			object_id: object_id.clone(),
			field_id: command1.field_id,
			current: vec![0, 1].as_slice().into(),
			new: vec![2, 0, 0].as_slice().into(),
			reset: None,
		};
		command2.execute(&mut room, user1_id).unwrap();
		assert_eq!(
			*room.get_object(&object_id).unwrap().get_structure(&command1.field_id).unwrap(),
			command1.new.as_slice()
		);

		let command3 = CompareAndSetStructureCommand {
			object_id: object_id.clone(),
			field_id: command1.field_id,
			current: command1.new,
			new: vec![3, 0, 0].as_slice().into(),
			reset: None,
		};
		command3.execute(&mut room, user1_id).unwrap();
		assert_eq!(
			*room.get_object(&object_id).unwrap().get_structure(&command1.field_id).unwrap(),
			command3.new.as_slice()
		);
	}

	///
	/// Проверяем что команда отсылает изменения другим клиентам
	///
	#[test]
	fn should_send_command() {
		let (mut room, user1_id, _, object_id, field_id) = setup();
		let command = CompareAndSetLongCommand {
			object_id,
			field_id,
			current: 0,
			new: 100,
			reset: Some(555),
		};

		room.test_out_commands.clear();
		command.execute(&mut room, user1_id).unwrap();
		assert!(
			matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetLong(c))) if
			c.value==command.new)
		);
	}

	///
	/// Проверяем что при выходе пользователя будет установлено заданное значение
	///
	#[test]
	fn should_reset() {
		let (mut room, user1_id, _, object_id, field_id) = setup();
		let command = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id,
			current: 0,
			new: 100,
			reset: Some(555),
		};
		command.execute(&mut room, user1_id).unwrap();
		assert_eq!(
			*room.get_object(&object_id).unwrap().get_long(&command.field_id).unwrap(),
			command.new
		);

		room.disconnect_user(user1_id).unwrap();
		assert_eq!(
			*room.get_object(&object_id).unwrap().get_long(&command.field_id).unwrap(),
			command.reset.unwrap()
		);
	}

	///
	/// Проверяем что при выходе пользователя не будет сброшено значение, если была вторая
	/// команда CompareAndSet без установки reset
	///
	#[test]
	fn should_disable_reset() {
		let (mut room, user1_id, _, object_id, field_id) = setup();
		CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id,
			current: 0,
			new: 100,
			reset: Some(555),
		}
		.execute(&mut room, user1_id)
		.unwrap();
		CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id,
			current: 100,
			new: 200,
			reset: None,
		}
		.execute(&mut room, user1_id)
		.unwrap();

		assert_eq!(*room.get_object(&object_id).unwrap().get_long(&field_id).unwrap(), 200);
		room.disconnect_user(user1_id).unwrap();
		assert_eq!(*room.get_object(&object_id).unwrap().get_long(&field_id).unwrap(), 200);
	}

	///
	/// Проверяем,что если два пользователя друг за другом поменяли значения,
	/// то при разрыве соединения первого пользователя данные не будут заменены
	///
	#[test]
	fn should_correct_reset_when_with_two_members() {
		let (mut room, user1_id, user2_id, object_id, field_id) = setup();
		let command_1 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id,
			current: 0,
			new: 100,
			reset: Some(555),
		};
		let command_2 = CompareAndSetLongCommand {
			object_id: object_id.clone(),
			field_id,
			current: 100,
			new: 200,
			reset: Some(1555),
		};
		command_1.execute(&mut room, user1_id).unwrap();
		command_2.execute(&mut room, user2_id).unwrap();

		room.disconnect_user(user1_id).unwrap();
		assert_eq!(
			*room.get_object(&object_id).unwrap().get_long(&command_1.field_id).unwrap(),
			command_2.new
		);
	}

	fn setup() -> (Room, RoomMemberId, RoomMemberId, GameObjectId, FieldId) {
		let access_group = AccessGroups(55);
		let mut template = RoomTemplate::default();
		let user_template_1 = MemberTemplate {
			private_key: Default::default(),
			groups: access_group,
			objects: Default::default(),
		};
		let user_template_2 = MemberTemplate {
			private_key: Default::default(),
			groups: access_group,
			objects: Default::default(),
		};

		let user_template_3 = MemberTemplate {
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
		let user1_id = room.register_member(user_template_1);
		let user2_id = room.register_member(user_template_2);
		let user3_id = room.register_member(user_template_3);
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(user3_id), access_group);
		object.created = true;
		object.template_id = object_template;

		let object_id = object.id.clone();
		(room, user1_id, user2_id, object_id, object_field)
	}
}
