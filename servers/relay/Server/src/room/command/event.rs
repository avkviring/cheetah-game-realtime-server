use cheetah_relay_common::commands::command::event::EventCommand;
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::ServerCommandExecutor;
use crate::room::object::GameObject;
use crate::room::template::config::Permission;
use crate::room::types::FieldType;
use crate::room::Room;

impl ServerCommandExecutor for EventCommand {
	fn execute(self, room: &mut Room, user_public_key: &UserPublicKey) {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();
		let action = |_object: &mut GameObject| Option::Some(S2CCommand::Event(self));
		room.do_action(&object_id, &field_id, FieldType::Event, user_public_key, Permission::Rw, action);
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::event::EventCommand;
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::RoomTemplate;
	use crate::room::tests::from_vec;
	use crate::room::Room;

	#[test]
	pub fn should_send_event() {
		let mut template = RoomTemplate::default();
		let user = template.create_user(1, AccessGroups(10));
		let mut room = Room::new_with_template(template);

		let object_id = room.create_object(&user).id.clone();
		room.out_commands.clear();

		let command = EventCommand {
			object_id: object_id.clone(),
			field_id: 100,
			event: from_vec(vec![1, 2, 3, 4, 5]),
		};
		command.clone().execute(&mut room, &user);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::Event(c))) if c==command));
	}

	#[test]
	pub fn should_not_panic_when_missing_object() {
		let mut template = RoomTemplate::default();
		let user = template.create_user(1, AccessGroups(10));
		let mut room = Room::new_with_template(template);

		let command = EventCommand {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 100,
			event: from_vec(vec![1, 2, 3, 4, 5]),
		};
		command.execute(&mut room, &user);
	}
}
