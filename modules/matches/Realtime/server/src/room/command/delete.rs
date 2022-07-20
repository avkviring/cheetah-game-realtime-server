use cheetah_matches_realtime_common::commands::types::delete::DeleteGameObjectCommand;
use cheetah_matches_realtime_common::room::owner::GameObjectOwner;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::Room;

impl ServerCommandExecutor for DeleteGameObjectCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let member = room.get_member(&member_id)?;
		if let GameObjectOwner::Member(object_id_user) = self.object_id.owner {
			if object_id_user != member.id {
				return Err(ServerCommandError::MemberNotOwnerGameObject {
					object_id: self.object_id.clone(),
					member_id,
				});
			}
		}
		room.delete_object(&self.object_id)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
	use cheetah_matches_realtime_common::commands::types::delete::DeleteGameObjectCommand;
	use cheetah_matches_realtime_common::room::access::AccessGroups;
	use cheetah_matches_realtime_common::room::owner::GameObjectOwner;

	use crate::room::command::{ServerCommandError, ServerCommandExecutor};
	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::room::Room;

	#[test]
	fn should_delete() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(0b11);

		let mut room = Room::from_template(template);
		let user_a_id = room.register_member(MemberTemplate::stub(access_groups));
		let user_b_id = room.register_member(MemberTemplate::stub(access_groups));
		room.test_mark_as_connected(user_a_id).unwrap();
		room.test_mark_as_connected(user_b_id).unwrap();

		let object_id = room
			.test_create_object_with_created_state(
				GameObjectOwner::Member(user_a_id),
				access_groups,
			)
			.id
			.clone();
		room.test_out_commands.clear();
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone(),
		};

		room.current_member_id = Option::Some(user_a_id);
		command.execute(&mut room, user_a_id).unwrap();

		assert!(matches!(room.get_object(&object_id), Err(_)));
		assert!(matches!(
			room.test_get_user_out_commands(user_a_id).pop_back(),
			None
		));
		assert!(
			matches!(room.test_get_user_out_commands(user_b_id).pop_back(), Some(S2CCommand::Delete(c)) if c==command)
		);
	}

	#[test]
	fn should_not_delete_if_not_owner() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(55);
		let mut room = Room::from_template(template);
		let user_a = room.register_member(MemberTemplate::stub(access_groups));
		let user_b = room.register_member(MemberTemplate::stub(access_groups));

		let object_id = room
			.test_create_object_with_not_created_state(
				GameObjectOwner::Member(user_a),
				access_groups,
			)
			.id
			.clone();
		room.test_out_commands.clear();
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone(),
		};

		assert!(matches!(
			command.execute(&mut room, user_b),
			Err(ServerCommandError::MemberNotOwnerGameObject {
				object_id: _,
				member_id: _
			})
		));
		assert!(matches!(room.get_object(&object_id), Ok(_)));
		assert!(matches!(room.test_out_commands.pop_back(), None));
	}
}
