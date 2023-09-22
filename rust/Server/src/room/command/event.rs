use cheetah_game_realtime_protocol::RoomMemberId;
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::event::{EventCommand, TargetEventCommand};
use cheetah_common::room::field::{Field, FieldType};
use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

impl ServerCommandExecutor for EventCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id;
		let action = |_object: &mut GameObject| Ok(Some(S2CCommand::Event(self.clone().into())));
		room.send_command_from_action(
			object_id,
			Field {
				id: field_id,
				field_type: FieldType::Long,
			},
			member_id,
			None,
			action,
		)
	}
}

impl ServerCommandExecutor for TargetEventCommand {
	fn execute(&self, room: &mut Room, member_id: u16) -> Result<(), ServerCommandError> {
		let field_id = self.event.field_id;
		let object_id = self.event.object_id;
		let target = self.target;
		let action = |_object: &mut GameObject| Ok(Some(S2CCommand::Event(self.event.clone().into())));
		room.send_command_from_action(
			object_id,
			Field {
				id: field_id,
				field_type: FieldType::Event,
			},
			member_id,
			Some(target),
			action,
		)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::event::{EventCommand, TargetEventCommand};
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::buffer::Buffer;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::room::command::tests::setup_one_player;
	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::room::Room;

	#[test]
	pub(crate) fn should_send_event() {
		let (mut room, member_id, access_groups) = setup_one_player();
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups);
		object.created = true;
		let object_id = object.id;
		room.test_out_commands.clear();

		let command = EventCommand {
			object_id,
			field_id: 100,
			event: Buffer::from(vec![1, 2, 3, 4, 5].as_slice()),
		};

		command.execute(&mut room, member_id).unwrap();
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::Event(c))) if c==command.into()));
	}

	#[test]
	pub(crate) fn should_send_event_to_member() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);

		let mut room = Room::from_template(template);
		let member1 = room.register_member(MemberTemplate::stub(access_groups));
		let member2 = room.register_member(MemberTemplate::stub(access_groups));
		let member3 = room.register_member(MemberTemplate::stub(access_groups));

		room.mark_as_connected_in_test(member1).unwrap();
		room.mark_as_connected_in_test(member2).unwrap();
		room.mark_as_connected_in_test(member3).unwrap();

		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member1), access_groups);
		object.created = true;
		let object_id = object.id;
		room.get_member_out_commands_for_test(member1).clear();
		room.get_member_out_commands_for_test(member2).clear();
		room.get_member_out_commands_for_test(member3).clear();

		let command = TargetEventCommand {
			target: member2,
			event: EventCommand {
				object_id,
				field_id: 100,
				event: Buffer::from(vec![1, 2, 3, 4, 5].as_slice()),
			},
		};

		command.execute(&mut room, member1).unwrap();
		assert!(matches!(room.get_member_out_commands_for_test(member1).pop_back(), None));
		assert!(matches!(room.get_member_out_commands_for_test(member2).pop_back(), Some(S2CCommand::Event(c)) if c.field_id == command.event.field_id));
		assert!(matches!(room.get_member_out_commands_for_test(member3).pop_back(), None));
	}
}
