use cheetah_matches_relay_common::commands::types::unload::DeleteGameObjectCommand;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::command::{error_c2s_command, ServerCommandExecutor};
use crate::room::Room;

impl ServerCommandExecutor for DeleteGameObjectCommand {
	fn execute(self, room: &mut Room, user_id: RoomMemberId) {
		let user = room.get_user(user_id).unwrap();
		if let GameObjectOwner::User(object_id_user) = self.object_id.owner {
			if object_id_user != user.id {
				error_c2s_command(
					"DeleteGameObjectCommand",
					room,
					user.id,
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
	use cheetah_matches_relay_common::commands::s2c::S2CCommand;
	use cheetah_matches_relay_common::commands::types::unload::DeleteGameObjectCommand;
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::{RoomTemplate, UserTemplate};
	use crate::room::Room;

	#[test]
	fn should_delete() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(0b11);

		let mut room = Room::from_template(template);
		let user_a_id = room.register_user(UserTemplate::stub(access_groups));
		let user_b_id = room.register_user(UserTemplate::stub(access_groups));
		room.mark_as_connected(user_a_id);
		room.mark_as_connected(user_b_id);

		let object_id = room.create_object(user_a_id, access_groups).id.clone();
		room.out_commands.clear();
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone(),
		};

		room.current_user = Option::Some(user_a_id);
		command.clone().execute(&mut room, user_a_id);
		
		assert!(matches!(room.get_object_mut(&object_id), None));
		assert!(matches!(room.get_user_out_commands(user_a_id).pop_back(), None));
		assert!(matches!(room.get_user_out_commands(user_b_id).pop_back(), Some(S2CCommand::Delete(c)) if c==command));
	}

	#[test]
	fn should_not_panic_when_missing_object() {
		let template = RoomTemplate::default();
		let mut room = Room::from_template(template);
		let user_id = room.register_user(UserTemplate::stub(AccessGroups(0b11)));

		let object_id = GameObjectId::new(100, GameObjectOwner::User(user_id));
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone(),
		};
		command.clone().execute(&mut room, user_id);
	}

	#[test]
	fn should_not_delete_if_not_owner() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(55);
		let mut room = Room::from_template(template);
		let user_a = room.register_user(UserTemplate::stub(access_groups));
		let user_b = room.register_user(UserTemplate::stub(access_groups));

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
