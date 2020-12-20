use cheetah_relay_common::commands::command::load::CreateGameObjectCommand;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::{error_c2s_command, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

impl ServerCommandExecutor for CreateGameObjectCommand {
	fn execute(self, room: &mut Room, user_public_key: &UserPublicKey) {
		let user = room.get_user(user_public_key).unwrap();

		if self.object_id.id == 0 {
			error_c2s_command(
				"CreateGameObjectCommand",
				room,
				&user.template.public_key,
				format!("0 is forbidden for game object id"),
			);
			return;
		}

		if !self.access_groups.is_sub_groups(&user.template.access_groups) {
			error_c2s_command(
				"CreateGameObjectCommand",
				room,
				&user.template.public_key,
				format!(
					"Incorrect access group {:?} with client groups {:?}",
					self.access_groups, user.template.access_groups
				),
			);
			return;
		}

		if let ObjectOwner::User(object_id_user) = self.object_id.owner {
			if object_id_user != user.template.public_key {
				error_c2s_command(
					"CreateGameObjectCommand",
					room,
					&user.template.public_key,
					format!("Incorrect object_id {:?} for user {:?}", self.object_id, user),
				);
				return;
			}
		}

		if room.contains_object(&self.object_id) {
			error_c2s_command(
				"CreateGameObjectCommand",
				room,
				&user.template.public_key,
				format!("Object already exists with id {:?}", self.object_id),
			);
			return;
		}

		let object = GameObject {
			id: self.object_id.clone(),
			template: self.template,
			access_groups: self.access_groups,
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
	use cheetah_relay_common::commands::command::load::CreateGameObjectCommand;
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::RoomTemplate;
	use crate::room::Room;

	#[test]
	fn should_create() {
		let mut template = RoomTemplate::default();
		let user_public_key = template.create_user(1, AccessGroups(0b11));
		let mut room = Room::from_template(template);

		let object_id = GameObjectId::new(1, ObjectOwner::User(user_public_key));
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b10),
		};
		command.clone().execute(&mut room, &user_public_key);

		assert!(matches!(
			room.get_object_mut(&object_id),
			Some(object)
				if object.template == command.template
				&& object.access_groups == command.access_groups
		));
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::Create(c))) if c==command));
	}

	///
	/// Проверяем что owner_id в идентификаторе объекта должен быть равен текущему
	///
	#[test]
	fn should_not_create_when_owner_in_object_id_is_wrong() {
		let mut template = RoomTemplate::default();
		let user_public_key = template.create_user(1, AccessGroups(0b11));
		let mut room = Room::from_template(template);

		let object_id = GameObjectId::new(1, ObjectOwner::User(1000));
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b10),
		};

		command.clone().execute(&mut room, &user_public_key);
		assert!(matches!(room.get_object_mut(&object_id), None));
		assert!(matches!(room.out_commands.pop_back(), None));
	}

	///
	/// AccessGroup нового объекта не должна быть больше чем группа клиента
	///
	#[test]
	fn should_not_create_when_access_group_is_wrong() {
		let mut template = RoomTemplate::default();
		let user_public_key = template.create_user(1, AccessGroups(0b11));
		let mut room = Room::from_template(template);

		let object_id = GameObjectId::new(1, ObjectOwner::User(user_public_key));
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b1000),
		};

		command.clone().execute(&mut room, &user_public_key);
		assert!(matches!(room.get_object_mut(&object_id), None));
		assert!(matches!(room.out_commands.pop_back(), None));
	}

	///
	/// AccessGroup нового объекта не должна быть больше чем группа клиента
	///
	#[test]
	fn should_not_create_when_id_is_zero() {
		let mut template = RoomTemplate::default();
		let user_public_key = template.create_user(1, AccessGroups(0b11));
		let mut room = Room::from_template(template);

		let object_id = GameObjectId::new(0, ObjectOwner::User(user_public_key));
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b11),
		};

		command.clone().execute(&mut room, &user_public_key);
		assert!(matches!(room.get_object_mut(&object_id), None));
		assert!(matches!(room.out_commands.pop_back(), None));
	}

	//
	/// Если есть объект - то он не должен быть замещен
	///
	#[test]
	fn should_not_replace_exists_object() {
		let mut template = RoomTemplate::default();
		let access_groups = AccessGroups(0b11);
		let user_public_key = template.create_user(1, access_groups);
		let mut room = Room::from_template(template);

		let object = room.create_object(&user_public_key, access_groups);
		object.template = 777;
		let object_id = object.id.clone();

		room.out_commands.clear();

		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b1000),
		};

		command.clone().execute(&mut room, &user_public_key);

		assert!(matches!(room.get_object_mut(&object_id), Some(object) if object.template == 777));
		assert!(matches!(room.out_commands.pop_back(), None));
	}
}
