use core::slice;
use std::cell::RefCell;
use std::rc::Rc;

use cheetah_matches_realtime_common::commands::field::{Field, FieldId};
use cheetah_matches_realtime_common::commands::s2c::S2CCommandWithMeta;
use cheetah_matches_realtime_common::commands::{
	s2c::S2CCommand,
	types::{long::CompareAndSetLongCommand, structure::CompareAndSetStructureCommand},
};
use cheetah_matches_realtime_common::commands::{FieldType, FieldValue};
use cheetah_matches_realtime_common::room::object::GameObjectId;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::template::config::Permission;
use crate::room::Room;

pub type CASCleanersStore = heapless::FnvIndexMap<(GameObjectId, FieldId, FieldType), FieldValue, 256>;

impl ServerCommandExecutor for CompareAndSetLongCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		perform_compare_and_set(
			room,
			member_id,
			self.object_id,
			self.field_id,
			&FieldValue::Long(self.current),
			FieldValue::Long(self.new),
			&self.reset.as_ref().map(|r| FieldValue::Long(*r)),
		)
	}
}

impl ServerCommandExecutor for CompareAndSetStructureCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		perform_compare_and_set(
			room,
			member_id,
			self.object_id,
			self.field_id,
			&FieldValue::Structure(self.current.as_slice().into()),
			FieldValue::Structure(self.new.as_slice().into()),
			&self.reset.as_ref().map(|r| FieldValue::Structure(r.as_slice().into())),
		)
	}
}

#[allow(clippy::map_err_ignore)]
pub fn perform_compare_and_set(
	room: &mut Room,
	member_id: RoomMemberId,
	object_id: GameObjectId,
	field_id: u16,
	current: &FieldValue,
	new: FieldValue,
	reset: &Option<FieldValue>,
) -> Result<(), ServerCommandError> {
	let field_type = current.field_type();
	let is_field_changed = Rc::new(RefCell::new(false));
	let action = |object: &mut GameObject| {
		let allow = match object.get_field_wrapped(field_id, field_type) {
			None => true,
			Some(value) => value == current,
		};
		if allow {
			*is_field_changed.borrow_mut() = true;
			object.set_field_wrapped(field_id, new.clone())?;
			if reset.is_some() {
				object.set_compare_and_set_owner(field_id, member_id)?;
			}
			Ok(Some(S2CCommand::new_set_command(new, object.id, field_id)))
		} else {
			Ok(None)
		}
	};

	room.send_command_from_action(object_id, Field { id: field_id, field_type }, member_id, Permission::Rw, None, action)?;

	if is_field_changed.take() {
		let m = room.get_member_mut(&member_id)?;
		let cls = &mut m.compare_and_set_cleaners;
		match &reset {
			None => {
				cls.remove(&(object_id, field_id, field_type));
			}
			Some(reset_value) => {
				cls.insert((object_id, field_id, field_type), reset_value.clone())
					.map_err(|_| ServerCommandError::Error("CompareAndSetCleaners overflow".to_owned()))?;
			}
		}
	}

	Ok(())
}

pub fn reset_all_compare_and_set(
	room: &mut Room,
	member_id: RoomMemberId,
	compare_and_set_cleaners: &CASCleanersStore,
) -> Result<(), ServerCommandError> {
	for ((object_id, field, _), reset) in compare_and_set_cleaners {
		apply_reset(room, member_id, *object_id, *field, reset)?;
	}

	Ok(())
}

pub fn apply_reset(
	room: &mut Room,
	member_id: RoomMemberId,
	object_id: GameObjectId,
	field: FieldId,
	reset: &FieldValue,
) -> Result<(), ServerCommandError> {
	match room.get_object_mut(object_id) {
		Err(_) => {
			// нормальная ситуация для пользовательских объектов
		}
		Ok(object) => {
			if let Some(owner) = object.get_compare_and_set_owner(&field) {
				if *owner == member_id {
					let command = reset_value(object, field, reset, member_id)?;
					let groups = object.access_groups;
					let template = object.template_id;
					room.send_to_members(groups, Some(template), slice::from_ref(&command), |_| true)?;
				}
			}
		}
	};

	Ok(())
}

fn reset_value(
	object: &mut GameObject,
	field_id: FieldId,
	value: &FieldValue,
	creator: RoomMemberId,
) -> Result<S2CCommandWithMeta, ServerCommandError> {
	object.set_field_wrapped(field_id, value.clone())?;
	let command = S2CCommandWithMeta {
		field: Some(Field {
			id: field_id,
			field_type: value.field_type(),
		}),
		creator,
		command: S2CCommand::new_set_command(value.clone(), object.id, field_id),
	};

	Ok(command)
}

#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::commands::field::FieldId;
	use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
	use cheetah_matches_realtime_common::commands::types::long::CompareAndSetLongCommand;
	use cheetah_matches_realtime_common::commands::types::structure::CompareAndSetStructureCommand;
	use cheetah_matches_realtime_common::commands::FieldType;
	use cheetah_matches_realtime_common::room::access::AccessGroups;
	use cheetah_matches_realtime_common::room::object::GameObjectId;
	use cheetah_matches_realtime_common::room::owner::GameObjectOwner;
	use cheetah_matches_realtime_common::room::RoomMemberId;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::{
		GameObjectTemplatePermission, GroupsPermissionRule, MemberTemplate, Permission, PermissionField, RoomTemplate,
	};
	use crate::room::Room;
	use cheetah_matches_realtime_common::commands::field::Field;

	///
	/// Проверяем что при выполнении нескольких команд соблюдаются гарантии `CompareAndSet`
	///
	#[test]
	fn should_compare_and_set_long() {
		let (mut room, member1_id, _, object_id, field_id) = setup();
		let command1 = CompareAndSetLongCommand {
			object_id,
			field_id,
			current: 0,
			new: 100,
			reset: None,
		};
		command1.execute(&mut room, member1_id).unwrap();
		assert_eq!(
			*room.get_object_mut(object_id).unwrap().get_field::<i64>(command1.field_id).unwrap(),
			command1.new
		);

		let command2 = CompareAndSetLongCommand {
			object_id,
			field_id: command1.field_id,
			current: 0,
			new: 200,
			reset: None,
		};
		command2.execute(&mut room, member1_id).unwrap();
		assert_eq!(
			*room.get_object_mut(object_id).unwrap().get_field::<i64>(command1.field_id).unwrap(),
			command1.new
		);

		let command3 = CompareAndSetLongCommand {
			object_id,
			field_id: command1.field_id,
			current: command1.new,
			new: 300,
			reset: None,
		};
		command3.execute(&mut room, member1_id).unwrap();
		assert_eq!(
			*room.get_object_mut(object_id).unwrap().get_field::<i64>(command1.field_id).unwrap(),
			command3.new
		);
	}

	#[test]
	fn should_compare_and_set_structure() {
		let (mut room, member1_id, _, object_id, field_id) = setup();
		let command1 = CompareAndSetStructureCommand {
			object_id,
			field_id,
			current: vec![0, 1].as_slice().into(),
			new: vec![1, 0, 0].as_slice().into(),
			reset: None,
		};
		command1.execute(&mut room, member1_id).unwrap();
		assert_eq!(
			*room.get_object_mut(object_id).unwrap().get_field::<Vec<u8>>(command1.field_id).unwrap(),
			command1.new.as_slice()
		);

		let command2 = CompareAndSetStructureCommand {
			object_id,
			field_id: command1.field_id,
			current: vec![0, 1].as_slice().into(),
			new: vec![2, 0, 0].as_slice().into(),
			reset: None,
		};
		command2.execute(&mut room, member1_id).unwrap();
		assert_eq!(
			*room.get_object_mut(object_id).unwrap().get_field::<Vec<u8>>(command1.field_id).unwrap(),
			command1.new.as_slice()
		);

		let command3 = CompareAndSetStructureCommand {
			object_id,
			field_id: command1.field_id,
			current: command1.new,
			new: vec![3, 0, 0].as_slice().into(),
			reset: None,
		};
		command3.execute(&mut room, member1_id).unwrap();
		assert_eq!(
			*room.get_object_mut(object_id).unwrap().get_field::<Vec<u8>>(command1.field_id).unwrap(),
			command3.new.as_slice()
		);
	}

	///
	/// Проверяем что команда отсылает изменения другим клиентам
	///
	#[test]
	fn should_send_command() {
		let (mut room, member1_id, _, object_id, field_id) = setup();
		let command = CompareAndSetLongCommand {
			object_id,
			field_id,
			current: 0,
			new: 100,
			reset: Some(555),
		};

		room.test_out_commands.clear();
		command.execute(&mut room, member1_id).unwrap();
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetField(c))) if
			c.value==command.new.into()));
	}

	///
	/// Проверяем что при выходе пользователя будет установлено заданное значение
	///
	#[test]
	fn should_reset() {
		let (mut room, member1_id, _, object_id, field_id) = setup();
		let command = CompareAndSetLongCommand {
			object_id,
			field_id,
			current: 0,
			new: 100,
			reset: Some(555),
		};
		command.execute(&mut room, member1_id).unwrap();
		assert_eq!(
			*room.get_object_mut(object_id).unwrap().get_field::<i64>(command.field_id).unwrap(),
			command.new
		);

		room.disconnect_member(member1_id).unwrap();
		assert_eq!(
			*room.get_object_mut(object_id).unwrap().get_field::<i64>(command.field_id).unwrap(),
			command.reset.unwrap()
		);
	}

	///
	/// Проверяем что при выходе пользователя не будет сброшено значение, если была вторая
	/// команда `CompareAndSet` без установки reset
	///
	#[test]
	fn should_disable_reset() {
		let (mut room, member1_id, _, object_id, field_id) = setup();
		CompareAndSetLongCommand {
			object_id,
			field_id,
			current: 0,
			new: 100,
			reset: Some(555),
		}
		.execute(&mut room, member1_id)
		.unwrap();
		CompareAndSetLongCommand {
			object_id,
			field_id,
			current: 100,
			new: 200,
			reset: None,
		}
		.execute(&mut room, member1_id)
		.unwrap();

		assert_eq!(*room.get_object_mut(object_id).unwrap().get_field::<i64>(field_id).unwrap(), 200);
		room.disconnect_member(member1_id).unwrap();
		assert_eq!(*room.get_object_mut(object_id).unwrap().get_field::<i64>(field_id).unwrap(), 200);
	}

	///
	/// Проверяем,что если два пользователя друг за другом поменяли значения,
	/// то при разрыве соединения первого пользователя данные не будут заменены
	///
	#[test]
	fn should_correct_reset_when_with_two_members() {
		let (mut room, member1_id, member2_id, object_id, field_id) = setup();
		let command_1 = CompareAndSetLongCommand {
			object_id,
			field_id,
			current: 0,
			new: 100,
			reset: Some(555),
		};
		let command_2 = CompareAndSetLongCommand {
			object_id,
			field_id,
			current: 100,
			new: 200,
			reset: Some(1555),
		};
		command_1.execute(&mut room, member1_id).unwrap();
		command_2.execute(&mut room, member2_id).unwrap();

		room.disconnect_member(member1_id).unwrap();
		assert_eq!(
			*room.get_object_mut(object_id).unwrap().get_field::<i64>(command_1.field_id).unwrap(),
			command_2.new
		);
	}

	fn setup() -> (Room, RoomMemberId, RoomMemberId, GameObjectId, FieldId) {
		let access_group = AccessGroups(55);
		let mut template = RoomTemplate::default();
		let member_template_1 = MemberTemplate::new_member(access_group, Default::default());
		let member_template_2 = MemberTemplate::new_member(access_group, Default::default());
		let member_template_3 = MemberTemplate::new_member(access_group, Default::default());

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
		let member1_id = room.register_member(member_template_1);
		let member2_id = room.register_member(member_template_2);
		let member3_id = room.register_member(member_template_3);
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member3_id), access_group);
		object.created = true;
		object.template_id = object_template;

		let object_id = object.id;
		(room, member1_id, member2_id, object_id, object_field)
	}
}
