use cheetah_matches_relay_common::commands::types::load::CreateGameObjectCommand;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::command::{error_c2s_command, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

impl ServerCommandExecutor for CreateGameObjectCommand {
	fn execute(self, room: &mut Room, user_id: RoomMemberId) {
		let user = room.get_user(user_id).unwrap();

		if self.object_id.id == 0 {
			error_c2s_command(
				"CreateGameObjectCommand",
				room,
				user.id,
				format!("0 is forbidden for game object id"),
			);
			return;
		}

		let groups = self.access_groups;

		if !groups.is_sub_groups(&user.template.groups) {
			error_c2s_command(
				"CreateGameObjectCommand",
				room,
				user.id,
				format!(
					"Incorrect access group {:?} with client groups {:?}",
					groups, user.template.groups
				),
			);
			return;
		}

		if let GameObjectOwner::User(object_id_user) = self.object_id.owner {
			if object_id_user != user.id {
				error_c2s_command(
					"CreateGameObjectCommand",
					room,
					user.id,
					format!("Incorrect object_id {:?} for user {:?}", self.object_id, user),
				);
				return;
			}
		}

		if room.contains_object(&self.object_id) {
			error_c2s_command(
				"CreateGameObjectCommand",
				room,
				user.id,
				format!("Object already exists with id {:?}", self.object_id),
			);
			return;
		}

		let object = GameObject {
			id: self.object_id.clone(),
			template: self.template,
			access_groups: groups,
			created: false,
			longs: Default::default(),
			floats: Default::default(),
			structures: Default::default(),
			compare_and_set_owners: Default::default(),
		};

		room.insert_object(object);
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::types::load::CreateGameObjectCommand;
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::{RoomTemplate, UserTemplate};
	use crate::room::Room;

	#[test]
	fn should_create() {
		let (mut room, user_id) = setup(AccessGroups(0b11));
		room.mark_as_connected(user_id);

		let object_id = GameObjectId::new(1, GameObjectOwner::User(user_id));
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b10),
		};
		command.clone().execute(&mut room, user_id);

		assert!(matches!(
			room.get_object_mut(&object_id),
			Some(object)
				if object.template == command.template
				&& object.access_groups == command.access_groups
		));
	}

	///
	/// Проверяем что owner_id в идентификаторе объекта должен быть равен текущему
	///
	#[test]
	fn should_not_create_when_owner_in_object_id_is_wrong() {
		let (mut room, user_id) = setup(AccessGroups(0b11));

		let object_id = GameObjectId::new(1, GameObjectOwner::User(1000));
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b10),
		};

		command.clone().execute(&mut room, user_id);
		assert!(matches!(room.get_object_mut(&object_id), None));
	}

	///
	/// AccessGroup нового объекта не должна быть больше чем группа клиента
	///
	#[test]
	fn should_not_create_when_access_group_is_wrong() {
		let (mut room, user_id) = setup(AccessGroups(0b11));
		let object_id = GameObjectId::new(1, GameObjectOwner::User(user_id));
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b1000),
		};
		command.clone().execute(&mut room, user_id);
		assert!(matches!(room.get_object_mut(&object_id), None));
	}

	///
	/// AccessGroup нового объекта не должна быть больше чем группа клиента
	///
	#[test]
	fn should_not_create_when_id_is_zero() {
		let (mut room, user_id) = setup(AccessGroups(0b11));

		let object_id = GameObjectId::new(0, GameObjectOwner::User(user_id));
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b11),
		};

		command.clone().execute(&mut room, user_id);
		assert!(matches!(room.get_object_mut(&object_id), None));
	}

	//
	/// Если есть объект - то он не должен быть замещен
	///
	#[test]
	fn should_not_replace_exists_object() {
		let access_groups = AccessGroups(0b11);
		let (mut room, user_id) = setup(access_groups.clone());
		let object = room.create_object(user_id, access_groups);
		object.template = 777;
		let object_id = object.id.clone();
		room.out_commands.clear();
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b1000),
		};

		command.clone().execute(&mut room, user_id);

		assert!(matches!(room.get_object_mut(&object_id), Some(object) if object.template == 777));
	}

	fn setup(access_groups: AccessGroups) -> (Room, u16) {
		let template = RoomTemplate::default();
		let mut room = Room::from_template(template);
		let user_id = room.register_user(UserTemplate::stub(access_groups));
		(room, user_id)
	}
}
