use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
use cheetah_matches_realtime_common::commands::types::event::{EventCommand, TargetEventCommand};
use cheetah_matches_realtime_common::commands::FieldType;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::{Field, GameObject};
use crate::room::template::config::Permission;
use crate::room::Room;

impl ServerCommandExecutor for EventCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();
		let action = |_object: &mut GameObject| Ok(Some(S2CCommand::Event(self.clone())));
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
		)
	}
}

impl ServerCommandExecutor for TargetEventCommand {
	fn execute(&self, room: &mut Room, user_id: u16) -> Result<(), ServerCommandError> {
		let field_id = self.event.field_id;
		let object_id = self.event.object_id.clone();
		let target = self.target;
		let action = |_object: &mut GameObject| Ok(Some(S2CCommand::Event(self.event.clone())));
		room.send_command_from_action(
			&object_id,
			Field {
				id: field_id,
				field_type: FieldType::Event,
			},
			user_id,
			Permission::Rw,
			Option::Some(target),
			action,
		)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::commands::binary_value::BinaryValue;
	use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
	use cheetah_matches_realtime_common::commands::types::event::{EventCommand, TargetEventCommand};
	use cheetah_matches_realtime_common::room::access::AccessGroups;
	use cheetah_matches_realtime_common::room::owner::GameObjectOwner;

	use crate::room::command::tests::setup_one_player;
	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::room::Room;

	#[test]
	pub fn should_send_event() {
		let (mut room, user, access_groups) = setup_one_player();
		let object = room.test_create_object_with_not_created_state(
			GameObjectOwner::Member(user),
			access_groups,
		);
		object.created = true;
		let object_id = object.id.clone();
		room.test_out_commands.clear();

		let command = EventCommand {
			object_id,
			field_id: 100,
			event: BinaryValue::from(vec![1, 2, 3, 4, 5].as_slice()),
		};

		command.execute(&mut room, user).unwrap();
		assert!(
			matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::Event(c))) if c==command)
		);
	}

	#[test]
	pub fn should_send_event_to_user() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);

		let mut room = Room::from_template(template);
		let user1 = room.register_member(MemberTemplate::stub(access_groups));
		let user2 = room.register_member(MemberTemplate::stub(access_groups));
		let user3 = room.register_member(MemberTemplate::stub(access_groups));

		room.test_mark_as_connected(user1).unwrap();
		room.test_mark_as_connected(user2).unwrap();
		room.test_mark_as_connected(user3).unwrap();

		let object = room.test_create_object_with_not_created_state(
			GameObjectOwner::Member(user1),
			access_groups,
		);
		object.created = true;
		let object_id = object.id.clone();
		room.test_get_user_out_commands(user1).clear();
		room.test_get_user_out_commands(user2).clear();
		room.test_get_user_out_commands(user3).clear();

		let command = TargetEventCommand {
			target: user2,
			event: EventCommand {
				object_id,
				field_id: 100,
				event: BinaryValue::from(vec![1, 2, 3, 4, 5].as_slice()),
			},
		};

		command.execute(&mut room, user1).unwrap();
		assert!(matches!(
			room.test_get_user_out_commands(user1).pop_back(),
			None
		));
		assert!(
			matches!(room.test_get_user_out_commands(user2).pop_back(), Some(S2CCommand::Event(c)) if c.field_id == command.event.field_id)
		);
		assert!(matches!(
			room.test_get_user_out_commands(user3).pop_back(),
			None
		));
	}
}
