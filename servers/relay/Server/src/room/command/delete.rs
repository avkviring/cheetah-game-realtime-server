use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::UserId;

use crate::room::command::{error_c2s_command, ServerCommandExecutor};
use crate::room::Room;

impl ServerCommandExecutor for DeleteGameObjectCommand {
	fn execute(self, room: &mut Room, user_id: UserId) {
		let user = room.get_user(user_id).unwrap();
		if let ObjectOwner::User(object_id_user) = self.object_id.owner {
			if object_id_user != user.template.id {
				error_c2s_command(
					"DeleteGameObjectCommand",
					room,
					user.template.id,
					format!("User not owner for game object {:?} for user {:?}", self.object_id, user),
				);
				return;
			}
		}
		room.delete_object(&self.object_id);
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::RoomTemplate;
	use crate::room::Room;

	#[test]
	fn should_delete() {
		let mut template = RoomTemplate::default();
		let access_groups = AccessGroups(0b11);
		let user_id = 1;
		template.configure_user(user_id, access_groups);
		let mut room = Room::from_template(template);

		let object_id = room.create_object(user_id, access_groups).id.clone();
		room.out_commands.clear();
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone(),
		};

		command.clone().execute(&mut room, user_id);

		assert!(matches!(room.get_object_mut(&object_id), None));
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::Delete(c))) if c==command));
	}

	#[test]
	fn should_not_panic_when_missing_object() {
		let mut template = RoomTemplate::default();
		let user_id = 1;
		template.configure_user(user_id, AccessGroups(0b11));
		let mut room = Room::from_template(template);

		let object_id = GameObjectId::new(100, ObjectOwner::User(user_id));
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone(),
		};
		command.clone().execute(&mut room, user_id);
	}

	#[test]
	fn should_not_delete_if_not_owner() {
		let mut template = RoomTemplate::default();
		let access_groups = AccessGroups(55);
		let user_a = 1;
		let user_b = 2;
		template.configure_user(user_a, access_groups);
		template.configure_user(user_b, access_groups);
		let mut room = Room::from_template(template);

		let object_id = room.create_object(user_a, access_groups).id.clone();
		room.out_commands.clear();
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone(),
		};

		command.clone().execute(&mut room, user_b);

		assert!(matches!(room.get_object_mut(&object_id), Some(_)));
		assert!(matches!(room.out_commands.pop_back(), None));
	}
}
