use std::process::id;

use cheetah_relay_common::commands::command::load::CreateGameObjectCommand;
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::room::owner::ClientOwner;

use crate::room::{Room, User};
use crate::room::command::{error_c2s_command, ServerCommandExecutor};
use crate::room::object::GameObject;
use std::collections::HashMap;
use cheetah_relay_common::commands::hash::UserPublicKey;

impl ServerCommandExecutor for CreateGameObjectCommand {
	fn execute(self, room: &mut Room, user_public_key: &UserPublicKey) {
		let user = room.users.get(user_public_key).unwrap();
		if !self.access_groups.is_sub_groups(&user.access_groups) {
			error_c2s_command(
				"CreateGameObjectCommand",
				room,
				user,
				format!("Incorrect access group {:?} with client groups {:?}", self.access_groups, user.access_groups),
			);
			return;
		}
		
		if let ClientOwner::Client(object_id_user) = self.object_id.owner {
			if object_id_user != user.public_key {
				error_c2s_command(
					"CreateGameObjectCommand",
					room,
					user,
					format!("Incorrect object_id {:?} for user {:?}", self.object_id, user),
				);
				return;
			}
		}
		
		if room.objects.contains_key(&self.object_id) {
			error_c2s_command(
				"CreateGameObjectCommand",
				room,
				user,
				format!("Object already exists with id {:?}", self.object_id),
			);
			return;
		}
		
		let mut object = GameObject {
			id: self.object_id.clone(),
			template: self.template,
			access_groups: self.access_groups,
			fields: self.fields.clone(),
		};
		room.objects.insert(object.id.clone(), object);
		room.send(self.access_groups, S2CCommandUnion::Create(self));
	}
}