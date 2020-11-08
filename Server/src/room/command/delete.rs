use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
use cheetah_relay_common::commands::hash::UserPublicKey;
use cheetah_relay_common::room::owner::ClientOwner;

use crate::room::{Room, User};
use crate::room::command::{error_c2s_command, ServerCommandExecutor};

impl ServerCommandExecutor for DeleteGameObjectCommand {
	fn execute(self, room: &mut Room, user_public_key: &UserPublicKey) {
		let user = room.get_user(user_public_key).unwrap();
		if let ClientOwner::Client(object_id_user) = self.object_id.owner {
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
		if let Some(object) = room.remove_object(&self.object_id) {
			let access_groups = object.access_groups;
			room.send(access_groups, S2CCommandUnion::Delete(self));
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

