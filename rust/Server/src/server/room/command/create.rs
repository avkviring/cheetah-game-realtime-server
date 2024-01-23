use cheetah_game_realtime_protocol::RoomMemberId;

use cheetah_common::commands::types::create::CreateGameObject;
use cheetah_common::room::owner::GameObjectOwner;

use crate::server::room::command::ServerCommandError;
use crate::server::room::object::GameObject;
use crate::server::room::Room;

pub(crate) fn create_object(command: &CreateGameObject, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let member = room.get_member(&member_id)?;

	if command.object_id.id == 0 {
		return Err(ServerCommandError::Error("0 is forbidden for game object id".to_owned()));
	}

	let groups = command.access_groups;

	if !groups.is_sub_groups(&member.template.groups) {
		return Err(ServerCommandError::Error(format!(
			"Incorrect access group {:?} with client groups {:?}",
			groups, member.template.groups
		)));
	}

	if let GameObjectOwner::Member(object_id_member) = command.object_id.get_owner() {
		if object_id_member != member.id {
			return Err(ServerCommandError::Error(format!("Incorrect object_id {:?} for member {member:?}", command.object_id)));
		}
	}

	if room.contains_object(&command.object_id) {
		return Err(ServerCommandError::Error(format!("Object already exists with id {:?}", command.object_id)));
	}
	let config = room.get_object_config(&command.template);
	room.insert_object(GameObject::new(command.object_id, command.template, groups, config, false));
	Ok(())
}

#[cfg(test)]
mod tests {
	use crate::server::room::command::create::create_object;
	use cheetah_common::commands::types::create::CreateGameObject;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::server::room::command::ServerCommandError;
	use crate::server::room::config::member::MemberCreateParams;
	use crate::server::room::config::room::RoomCreateParams;
	use crate::server::room::Room;

	#[test]
	fn should_create() {
		let (mut room, member_id) = setup(AccessGroups(0b11));
		room.mark_as_connected_in_test(member_id).unwrap();

		let object_id = GameObjectId::new(1, GameObjectOwner::Member(member_id));
		let command = CreateGameObject {
			object_id,
			template: 100,
			access_groups: AccessGroups(0b10),
		};
		create_object(&command, &mut room, member_id).unwrap();

		assert!(matches!(
			room.get_object_mut(object_id),
			Ok(object)
				if object.template_id == command.template
				&& object.access_groups == command.access_groups
		));
	}

	///
	/// Проверяем что `owner_id` в идентификаторе объекта должен быть равен текущему
	///
	#[test]
	fn should_not_create_when_owner_in_object_id_is_wrong() {
		let (mut room, member_id) = setup(AccessGroups(0b11));

		let object_id = GameObjectId::new(1, GameObjectOwner::Member(1000));
		let command = CreateGameObject {
			object_id,
			template: 100,
			access_groups: AccessGroups(0b10),
		};

		assert!(matches!(create_object(&command, &mut room, member_id), Err(ServerCommandError::Error(_))));
		assert!(matches!(room.get_object_mut(object_id), Err(_)));
	}

	///
	/// `AccessGroup` нового объекта не должна быть больше чем группа клиента
	///
	#[test]
	fn should_not_create_when_access_group_is_wrong() {
		let (mut room, member_id) = setup(AccessGroups(0b11));
		let object_id = GameObjectId::new(1, GameObjectOwner::Member(member_id));
		let command = CreateGameObject {
			object_id,
			template: 100,
			access_groups: AccessGroups(0b1000),
		};

		assert!(matches!(create_object(&command, &mut room, member_id), Err(ServerCommandError::Error(_))));
		assert!(matches!(room.get_object_mut(object_id), Err(_)));
	}

	///
	/// `AccessGroup` нового объекта не должна быть больше чем группа клиента
	///
	#[test]
	fn should_not_create_when_id_is_zero() {
		let (mut room, member_id) = setup(AccessGroups(0b11));

		let object_id = GameObjectId::new(0, GameObjectOwner::Member(member_id));
		let command = CreateGameObject {
			object_id,
			template: 100,
			access_groups: AccessGroups(0b11),
		};
		assert!(matches!(create_object(&command, &mut room, member_id), Err(ServerCommandError::Error(_))));
		assert!(matches!(room.get_object_mut(object_id), Err(_)));
	}

	//
	/// Если есть объект - то он не должен быть замещен
	///
	#[test]
	fn should_not_replace_exists_object() {
		let access_groups = AccessGroups(0b11);
		let (mut room, member_id) = setup(access_groups);
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups, Default::default());
		object.template_id = 777;
		let object_id = object.id;
		room.test_out_commands.clear();
		let command = CreateGameObject {
			object_id,
			template: 100,
			access_groups: AccessGroups(0b1000),
		};

		assert!(matches!(create_object(&command, &mut room, member_id), Err(ServerCommandError::Error(_))));
		assert!(matches!(room.get_object_mut(object_id), Ok(object) if object.template_id == 777));
	}

	fn setup(access_groups: AccessGroups) -> (Room, u16) {
		let template = RoomCreateParams::default();
		let mut room = Room::new(0, template);
		let member_id = room.register_member(MemberCreateParams::stub(access_groups));
		(room, member_id)
	}
}
