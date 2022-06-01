#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::s2c::S2CCommand;
	use cheetah_matches_relay_common::commands::types::field::SetFieldCommand;
	use cheetah_matches_relay_common::commands::FieldType;
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::room::Room;

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
}
