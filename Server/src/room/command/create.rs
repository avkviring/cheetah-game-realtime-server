use cheetah_relay_common::commands::command::load::CreateGameObjectCommand;
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::room::UserPublicKey;
use cheetah_relay_common::room::owner::ClientOwner;

use crate::room::Room;
use crate::room::command::{error_c2s_command, ServerCommandExecutor};
use crate::room::object::GameObject;

impl ServerCommandExecutor for CreateGameObjectCommand {
	fn execute(self, room: &mut dyn Room, user_public_key: &UserPublicKey) {
		let user = room.get_user(user_public_key).unwrap();
		if !self.access_groups.is_sub_groups(&user.access_groups) {
			error_c2s_command(
				"CreateGameObjectCommand",
				room,
				&user.public_key,
				format!("Incorrect access group {:?} with client groups {:?}", self.access_groups, user.access_groups),
			);
			return;
		}
		
		if let ClientOwner::Client(object_id_user) = self.object_id.owner {
			if object_id_user != user.public_key {
				error_c2s_command(
					"CreateGameObjectCommand",
					room,
					&user.public_key,
					format!("Incorrect object_id {:?} for user {:?}", self.object_id, user),
				);
				return;
			}
		}
		
		if room.contains_object(&self.object_id) {
			error_c2s_command(
				"CreateGameObjectCommand",
				room,
				&user.public_key,
				format!("Object already exists with id {:?}", self.object_id),
			);
			return;
		}
		
		let object = GameObject {
			id: self.object_id.clone(),
			template: self.template,
			access_groups: self.access_groups,
			fields: self.fields.clone(),
		};
		room.insert_object(object);
		room.send_to_group(self.access_groups, S2CCommandUnion::Create(self));
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::load::CreateGameObjectCommand;
	use cheetah_relay_common::commands::command::S2CCommandUnion;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ClientOwner;
	
	use crate::room::command::ServerCommandExecutor;
	use crate::room::Room;
	use crate::room::tests::RoomStub;
	
	#[test]
	fn should_create() {
		let mut room = RoomStub::new();
		let user_public_key = room.create_user(AccessGroups(0b11));
		let object_id = GameObjectId::new(1, ClientOwner::Client(user_public_key));
		let mut command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b10),
			fields: Default::default(),
		};
		command.fields.longs.insert(0, 100);
		
		command.clone().execute(&mut room, &user_public_key);
		
		assert!(matches!(
			room.get_object(&object_id),
			Some(object)
				if object.template == command.template
					&& object.access_groups == command.access_groups
					&& object.fields == command.fields
		));
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommandUnion::Create(c))) if c==command));
	}
	
	///
	/// Проверяем что owner_id в идентификаторе объекта должен быть равен текущему
	///
	#[test]
	fn should_not_create_when_owner_in_object_id_is_wrong() {
		let mut room = RoomStub::new();
		let user_public_key = room.create_user(AccessGroups(0b11));
		let object_id = GameObjectId::new(1, ClientOwner::Client(1000));
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b10),
			fields: Default::default(),
		};
		
		command.clone().execute(&mut room, &user_public_key);
		assert!(matches!(room.get_object(&object_id), None));
		assert!(matches!(room.out_commands.pop_back(), None));
	}
	
	///
	/// AccessGroup нового объекта не должна быть больше чем группа клиента
	///
	#[test]
	fn should_not_create_when_access_group_is_wrong() {
		let mut room = RoomStub::new();
		let user_public_key = room.create_user(AccessGroups(0b11));
		let object_id = GameObjectId::new(1, ClientOwner::Client(user_public_key));
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b1000),
			fields: Default::default(),
		};
		
		command.clone().execute(&mut room, &user_public_key);
		assert!(matches!(room.get_object(&object_id), None));
		assert!(matches!(room.out_commands.pop_back(), None));
	}
	
	//
	/// Если есть объект - то он не должен быть замещен
	///
	#[test]
	fn should_not_replace_exists_object() {
		let mut room = RoomStub::new();
		let user_public_key = room.create_user(AccessGroups(0b11));
		let object = room.create_object(&user_public_key);
		object.template = 777;
		let object_id = object.id.clone();
		
		let command = CreateGameObjectCommand {
			object_id: object_id.clone(),
			template: 100,
			access_groups: AccessGroups(0b1000),
			fields: Default::default(),
		};
		
		command.clone().execute(&mut room, &user_public_key);
		
		assert!(matches!(room.get_object(&object_id), Some(object) if object.template == 777));
		assert!(matches!(room.out_commands.pop_back(), None));
	}
}