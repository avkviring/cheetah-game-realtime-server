use cheetah_matches_relay_common::commands::command::event::{EventCommand, TargetEventCommand};
use cheetah_matches_relay_common::commands::command::S2CCommand;
use cheetah_matches_relay_common::room::UserId;

use crate::room::command::ServerCommandExecutor;
use crate::room::object::GameObject;
use crate::room::template::config::Permission;
use crate::room::types::FieldType;
use crate::room::Room;

impl ServerCommandExecutor for EventCommand {
	fn execute(self, room: &mut Room, user_id: UserId) {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();
		let action = |_object: &mut GameObject| Option::Some(S2CCommand::Event(self));
		room.build_command_and_send(&object_id, &field_id, FieldType::Event, user_id, Permission::Rw, action);
	}
}

impl ServerCommandExecutor for TargetEventCommand {
	fn execute(self, room: &mut Room, user_id: u16) {
		let field_id = self.event.field_id;
		let object_id = self.event.object_id.clone();
		let target = self.target;
		let action = |_object: &mut GameObject| Option::Some(S2CCommand::Event(self.event));
		room.build_command_and_send_to_user(&object_id, &field_id, FieldType::Event, user_id, Permission::Rw, action, target);
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::command::event::{EventCommand, TargetEventCommand};
	use cheetah_matches_relay_common::commands::command::S2CCommand;
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::ObjectOwner;
	use cheetah_matches_relay_common::room::UserId;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::{RoomTemplate, UserTemplate};
	use crate::room::tests::from_vec;
	use crate::room::Room;

	#[test]
	pub fn should_send_event() {
		let (mut room, user, access_groups) = setup();
		let object = room.create_object(user, access_groups);
		object.created = true;
		let object_id = object.id.clone();
		room.out_commands.clear();

		let command = EventCommand {
			object_id: object_id.clone(),
			field_id: 100,
			event: from_vec(vec![1, 2, 3, 4, 5]),
		};
		command.clone().execute(&mut room, user);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::Event(c))) if c==command));
	}

	#[test]
	pub fn should_send_event_to_user() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);

		let mut room = Room::from_template(template);
		let user1 = room.register_user(UserTemplate::stub(access_groups));
		let user2 = room.register_user(UserTemplate::stub(access_groups));
		let user3 = room.register_user(UserTemplate::stub(access_groups));

		room.mark_as_connected(user1);
		room.mark_as_connected(user2);
		room.mark_as_connected(user3);

		let object = room.create_object(user1, access_groups);
		object.created = true;
		let object_id = object.id.clone();
		room.get_user_out_commands(user1).clear();
		room.get_user_out_commands(user2).clear();
		room.get_user_out_commands(user3).clear();

		let command = TargetEventCommand {
			target: user2,
			event: EventCommand {
				object_id: object_id.clone(),
				field_id: 100,
				event: from_vec(vec![1, 2, 3, 4, 5]),
			},
		};

		command.clone().execute(&mut room, user1);
		assert!(matches!(room.get_user_out_commands(user1).pop_back(), None));
		assert!(matches!(room.get_user_out_commands(user2).pop_back(), Some(S2CCommand::Event(c)) if c.field_id == command.event.field_id));
		assert!(matches!(room.get_user_out_commands(user3).pop_back(), None));
	}

	#[test]
	pub fn should_not_panic_when_missing_object() {
		let (mut room, user, _) = setup();

		let command = EventCommand {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 100,
			event: from_vec(vec![1, 2, 3, 4, 5]),
		};
		command.execute(&mut room, user);
	}

	fn setup() -> (Room, UserId, AccessGroups) {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::from_template(template);
		let user_id = room.register_user(UserTemplate::stub(access_groups));
		(room, user_id, access_groups)
	}
}
