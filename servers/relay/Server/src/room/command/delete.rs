use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::{error_c2s_command, ServerCommandExecutor};
use crate::room::Room;

impl ServerCommandExecutor for DeleteGameObjectCommand {
	fn execute(self, room: &mut Room, user_public_key: &UserPublicKey) {
		let user = room.get_user(user_public_key).unwrap();
		if let ObjectOwner::User(object_id_user) = self.object_id.owner {
			if object_id_user != user.template.public_key {
				error_c2s_command(
					"DeleteGameObjectCommand",
					room,
					&user.template.public_key,
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
		let user_public_key = template.create_user(1, access_groups);
		let mut room = Room::from_template(template);

		let object_id = room.create_object(&user_public_key, access_groups).id.clone();
		room.out_commands.clear();
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone(),
		};

		command.clone().execute(&mut room, &user_public_key);

		assert!(matches!(room.get_object_mut(&object_id), None));
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::Delete(c))) if c==command));
	}

	#[test]
	fn should_not_panic_when_missing_object() {
		let mut template = RoomTemplate::default();
		let user_public_key = template.create_user(1, AccessGroups(0b11));
		let mut room = Room::from_template(template);

		let object_id = GameObjectId::new(100, ObjectOwner::User(user_public_key));
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone(),
		};
		command.clone().execute(&mut room, &user_public_key);
	}

	#[test]
	fn should_not_delete_if_not_owner() {
		let mut template = RoomTemplate::default();
		let access_groups = AccessGroups(55);
		let user_a = template.create_user(1, access_groups);
		let user_b = template.create_user(2, access_groups);
		let mut room = Room::from_template(template);

		let object_id = room.create_object(&user_a, access_groups).id.clone();
		room.out_commands.clear();
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone(),
		};

		command.clone().execute(&mut room, &user_b);

		assert!(matches!(room.get_object_mut(&object_id), Some(_)));
		assert!(matches!(room.out_commands.pop_back(), None));
	}
}
