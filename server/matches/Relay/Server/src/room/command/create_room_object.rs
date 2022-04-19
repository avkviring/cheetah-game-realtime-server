use cheetah_matches_relay_common::commands::types::create::C2SCreateRoomGameObjectCommand;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

///
/// Игровые объекты с владельцем комната создаются с временным пользовательским идентификатором,
/// так как клиент не может выдавать комнатные идентификаторы для объектов.
///
impl ServerCommandExecutor for C2SCreateRoomGameObjectCommand {
	fn execute(&self, room: &mut Room, _user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		room.room_object_id_generator += 1;
		let object_id = GameObjectId::new(room.room_object_id_generator, GameObjectOwner::Room);
		let object = GameObject::new(object_id.clone(), self.template, self.access_groups, false);
		room.insert_object(object);
		room.add_creating_object_id_mapping(self.temporary_object_id.clone(), object_id);
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::types::create::{
		C2SCreateRoomGameObjectCommand,
	};
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;

	use crate::room::command::{ServerCommandExecutor};
	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::room::Room;

	#[test]
	fn should_create() {
		let (mut room, user_id) = setup(AccessGroups(0b11));
		room.test_mark_as_connected(user_id).unwrap();

		let object_id = GameObjectId::new(1, GameObjectOwner::Member(user_id));
		let command = C2SCreateRoomGameObjectCommand {
			temporary_object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b10),
			unique_create_key: None,
		};
		command.execute(&mut room, user_id).unwrap();

		assert!(matches!(
			room.get_object(&object_id),
			Ok(object)
				if object.template_id == command.template
				&& object.access_groups == command.access_groups
				&& object.id.owner==GameObjectOwner::Room
		));
	}

	fn setup(access_groups: AccessGroups) -> (Room, u16) {
		let template = RoomTemplate::default();
		let mut room = Room::from_template(template);
		let user_id = room.register_member(MemberTemplate::stub(access_groups));
		(room, user_id)
	}
}
