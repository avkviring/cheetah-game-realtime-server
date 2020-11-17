use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
use cheetah_relay_common::room::UserPublicKey;
use cheetah_relay_common::room::owner::ObjectOwner;

use crate::room::command::{error_c2s_command, ServerCommandExecutor};
use crate::room::Room;

impl ServerCommandExecutor for DeleteGameObjectCommand {
	fn execute(self, room: &mut Room, user_public_key: &UserPublicKey) {
		let user = room.get_user(user_public_key).unwrap();
		if let ObjectOwner::User(object_id_user) = self.object_id.owner {
			if object_id_user != user.public_key {
				error_c2s_command(
					"DeleteGameObjectCommand",
					room,
					&user.public_key,
					format!("User not owner for game object {:?} for user {:?}", self.object_id, user),
				);
				return;
			}
		}
		
		let user_public_key = user.public_key.clone();
		if let Some(object) = room.delete_object(&self.object_id) {
			let access_groups = object.access_groups;
			room.send_to_group(access_groups, S2CCommand::Delete(self));
		} else {
			error_c2s_command(
				"DeleteGameObjectCommand",
				room,
				&user_public_key,
				format!("game object not found {:?}", self.object_id),
			);
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;
	
	use crate::room::command::ServerCommandExecutor;
	use crate::room::Room;
	
	#[test]
	fn should_delete() {
		let mut room = Room::new(0);
		let user_public_key = room.create_user(AccessGroups(55));
		let object_id = room.create_object(&user_public_key).id.clone();
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone()
		};
		
		command.clone().execute(&mut room, &user_public_key);
		
		assert!(matches!(room.get_object(&object_id), None));
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::Delete(c))) if c==command));
	}
	
	#[test]
	fn should_not_panic_when_missing_object() {
		let mut room = Room::new(0);
		let user_public_key = room.create_user(AccessGroups(55));
		let object_id = GameObjectId::new(100, ObjectOwner::User(user_public_key));
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone()
		};
		command.clone().execute(&mut room, &user_public_key);
	}
	
	#[test]
	fn should_not_delete_if_not_owner() {
		let mut room = Room::new(0);
		let user_a = room.create_user(AccessGroups(55));
		let user_b = room.create_user(AccessGroups(55));
		let object_id = room.create_object(&user_a).id.clone();
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone()
		};
		
		command.clone().execute(&mut room, &user_b);
		
		assert!(matches!(room.get_object(&object_id), Some(_)));
		assert!(matches!(room.out_commands.pop_back(), None));
	}
}