#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
	use cheetah_matches_realtime_common::commands::s2c::S2CCommandWithCreator;
	use cheetah_matches_realtime_common::commands::types::field::SetFieldCommand;
	use cheetah_matches_realtime_common::commands::FieldType;
	use cheetah_matches_realtime_common::protocol::frame::applications::BothDirectionCommand;
	use cheetah_matches_realtime_common::room::access::AccessGroups;
	use cheetah_matches_realtime_common::room::object::GameObjectId;
	use cheetah_matches_realtime_common::room::owner::GameObjectOwner;
	use cheetah_matches_realtime_common::room::RoomMemberId;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::object::Field;
	use crate::room::template::config::{
		GameObjectTemplatePermission, GroupsPermissionRule, MemberTemplate, Permission, PermissionField, RoomTemplate,
	};
	use crate::room::Room;

	const FIELD_ID: u16 = 100;

	#[test]
	pub fn should_set_structure() {
		let template = RoomTemplate::default();
		let mut room = Room::from_template(template);
		let access_groups = AccessGroups(10);
		let user = room.register_member(MemberTemplate::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(user), access_groups);
		object.created = true;
		let object_id = object.id.clone();

		room.test_out_commands.clear();
		let command = SetFieldCommand {
			object_id: object_id.clone(),
			field_id: 100,
			value: vec![1, 2, 3, 4, 5].into(),
		};

		command.execute(&mut room, user).unwrap();
		let object = room.get_object(&object_id).unwrap();

		assert_eq!(*object.get_field_wrapped(100, FieldType::Structure).unwrap(), command.value);
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetField(c))) if c == command));
	}

	fn init_set_structure_test() -> (Room, RoomMemberId, RoomMemberId, GameObjectId) {
		let access_groups = AccessGroups(10);
		let mut template = RoomTemplate::default();
		template.permissions.templates.push(GameObjectTemplatePermission {
			template: 0,
			rules: vec![GroupsPermissionRule {
				groups: access_groups,
				permission: Permission::Ro,
			}],
			fields: vec![PermissionField {
				field: Field {
					id: FIELD_ID,
					field_type: FieldType::Structure,
				},
				rules: vec![GroupsPermissionRule {
					groups: access_groups,
					permission: Permission::Rw,
				}],
			}],
		});
		let mut room = Room::from_template(template);
		let user1 = room.register_member(MemberTemplate::stub(access_groups));
		room.test_mark_as_connected(user1).unwrap();
		let user2 = room.register_member(MemberTemplate::stub(access_groups));
		room.test_mark_as_connected(user2).unwrap();
		let object1 = room.test_create_object_with_not_created_state(GameObjectOwner::Member(user1), access_groups);
		object1.created = true;
		let object_id1 = object1.id.clone();
		(room, user1, user2, object_id1)
	}

	fn run_set_structure_test(room: &mut Room, user1: RoomMemberId, user2: RoomMemberId, object_id: GameObjectId, sender: RoomMemberId) {
		let command = SetFieldCommand {
			object_id: object_id.clone(),
			field_id: FIELD_ID,
			value: vec![1, 2, 3, 4, 5].into(),
		};

		command.execute(room, sender).unwrap();
		let object = room.get_object(&object_id).unwrap();

		assert_eq!(*object.get_field_wrapped(FIELD_ID, FieldType::Structure).unwrap(), command.value);

		let _expected = S2CCommandWithCreator {
			command: S2CCommand::SetField(command),
			creator: user1,
		};

		let member1 = room.get_member(&user1).unwrap();
		assert!(matches!(
			member1.out_commands[0].command.clone(),
			BothDirectionCommand::S2CWithCreator(_expected)
		));
		let member2 = room.get_member(&user2).unwrap();
		assert!(matches!(
			member2.out_commands[0].command.clone(),
			BothDirectionCommand::S2CWithCreator(_expected)
		));
	}

	#[test]
	pub fn should_send_command_to_all_when_owner_sets_structure_field() {
		let (mut room, user1, user2, object_id) = init_set_structure_test();
		run_set_structure_test(&mut room, user1, user2, object_id, user1);
	}

	#[test]
	pub fn should_send_command_to_all_when_non_owner_sets_structure_field() {
		let (mut room, user1, user2, object_id) = init_set_structure_test();
		run_set_structure_test(&mut room, user1, user2, object_id, user2);
	}
}
