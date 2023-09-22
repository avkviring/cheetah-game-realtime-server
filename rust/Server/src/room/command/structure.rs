use cheetah_game_realtime_protocol::RoomMemberId;
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::structure::SetStructureCommand;
use cheetah_common::room::field::{Field, FieldType};
use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

impl ServerCommandExecutor for SetStructureCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id;

		let action = |object: &mut GameObject| {
			object.structures.set(self.field_id, self.value.clone());
			Ok(Some(S2CCommand::SetStructure(self.clone().into())))
		};

		room.send_command_from_action(
			object_id,
			Field {
				id: field_id,
				field_type: FieldType::Structure,
			},
			member_id,
			None,
			action,
		)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::structure::SetStructureCommand;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::buffer::Buffer;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::MemberTemplate;
	use crate::room::template::config::RoomTemplate;
	use crate::room::Room;

	#[test]
	pub(crate) fn should_set_structure() {
		let template = RoomTemplate::default();
		let mut room = Room::from_template(template);
		let access_groups = AccessGroups(10);
		let member_id = room.register_member(MemberTemplate::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups);
		object.created = true;
		let object_id = object.id;

		room.test_out_commands.clear();
		let command = SetStructureCommand {
			object_id,
			field_id: 100,
			value: Buffer::from(vec![1, 2, 3, 4, 5].as_slice()).into(),
		};

		command.execute(&mut room, member_id).unwrap();
		let object = room.get_object_mut(object_id).unwrap();

		assert_eq!(*object.structures.get(100).unwrap(), command.value);
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetStructure(c))) if c == 
			command.into()));
	}
}
