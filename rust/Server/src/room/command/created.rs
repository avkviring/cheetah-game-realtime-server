use cheetah_common::commands::types::create::C2SCreatedGameObjectCommand;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::owner::GameObjectOwner;
use cheetah_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::CreateCommandsCollector;
use crate::room::Room;

impl ServerCommandExecutor for C2SCreatedGameObjectCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let room_id = room.id;
		let object = room.get_object_mut(self.object_id)?;

		if object.created {
			return Err(ServerCommandError::Error(format!(
				"room[({:?})] object ({:?}) already created",
				room_id, object.id
			)));
		}

		let member_object_id = object.id;

		let object = if self.room_owner {
			// создаем объект с владением комнаты
			let new_room_object_id = GameObjectId::new(room.room_object_id_generator, GameObjectOwner::Room);
			if let Some(unique_key) = &self.singleton_key {
				if room.has_object_singleton_key(unique_key) {
					room.delete_object(member_object_id, member_id)?;
					return Ok(());
				}
				room.set_singleton_key(unique_key.clone(), new_room_object_id);
			}
			room.room_object_id_generator += 1;
			let mut object = room.delete_object(member_object_id, member_id)?;
			object.id = new_room_object_id;
			room.insert_object(object);
			room.get_object_mut(new_room_object_id)?
		} else {
			object
		};

		let groups = object.access_groups;
		object.created = true;
		// объект полностью загружен - теперь его надо загрузить остальным клиентам
		let mut commands = CreateCommandsCollector::new();
		object.collect_create_commands(&mut commands, member_id);
		let template = object.template_id;
		if object.id.get_owner() == GameObjectOwner::Room {
			room.send_to_members(groups, Some(template), commands.as_slice(), |_| true)?;
		} else {
			room.send_to_members(groups, Some(template), commands.as_slice(), |member| member.id != member_id)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::binary_value::BinaryValue;
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::create::{C2SCreatedGameObjectCommand, CreateGameObjectCommand};
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::room::command::tests::{setup_one_player, setup_two_players};
	use crate::room::command::{ServerCommandError, ServerCommandExecutor};

	///
	/// - Команда должна приводить к рассылки оповещения для пользователей
	/// - Команда не должна отсылаться обратно пользователю
	///
	#[test]
	pub(crate) fn should_send_commands() {
		let (mut room, object_id, member1, member2) = setup_two_players();
		room.test_mark_as_connected(member1).unwrap();
		room.test_mark_as_connected(member2).unwrap();
		let command = C2SCreatedGameObjectCommand {
			object_id,
			room_owner: false,
			singleton_key: None,
		};
		command.execute(&mut room, member1).unwrap();

		assert!(room.test_get_member_out_commands(member1).is_empty());
		assert!(matches!(
			room.test_get_member_out_commands(member2).get(0),
			Some(S2CCommand::Create(c)) if c.object_id == object_id
		));

		assert!(matches!(
			room.test_get_member_out_commands(member2).get(1),
			Some(S2CCommand::Created(c)) if c.object_id == object_id
		));
	}

	///
	/// Команда должна отметить объект как загруженный
	///
	#[test]
	pub(crate) fn should_switch_object_to_created_state() {
		let (mut room, object_id, member1, _) = setup_two_players();
		let command = C2SCreatedGameObjectCommand {
			object_id,
			room_owner: false,
			singleton_key: None,
		};
		room.test_out_commands.clear();
		command.execute(&mut room, member1).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		assert!(object.created);
	}

	///
	/// Не рассылать команду если объект уже создан
	/// Такого быть не должно, однако проверить стоит, так как команду могут послать умышленно.
	///
	#[test]
	pub(crate) fn should_dont_send_command_if_object_already_created() {
		let (mut room, object_id, member1, _) = setup_two_players();
		let object = room.get_object_mut(object_id).unwrap();
		object.created = true;
		let command = C2SCreatedGameObjectCommand {
			object_id,
			room_owner: false,
			singleton_key: None,
		};
		room.test_out_commands.clear();

		assert!(matches!(command.execute(&mut room, member1), Err(ServerCommandError::Error(_))));
		assert!(matches!(room.test_out_commands.pop_back(), None));
	}

	///
	/// Если создается объект с owner = room, то его id должен сменится на id с owner = room
	///
	#[test]
	pub(crate) fn should_convert_object_to_room_object() {
		let (mut room, member_id, access_groups) = setup_one_player();
		let member_object_id = GameObjectId::new(100, GameObjectOwner::Member(member_id));
		let create_command = CreateGameObjectCommand {
			object_id: member_object_id,
			template: 777,
			access_groups,
		};
		create_command.execute(&mut room, member_id).unwrap();

		let created_command = C2SCreatedGameObjectCommand {
			object_id: member_object_id,
			room_owner: true,
			singleton_key: None,
		};
		created_command.execute(&mut room, member_id).unwrap();

		// старого объекта уже не должно быть
		room.get_object_mut(member_object_id).unwrap_err();

		let (_object_id, object) = room.objects.first().unwrap();
		// это именно тот объект, который мы создали?
		assert_eq!(object.template_id, 777);
		// владелец должен быть комнатой
		assert_eq!(object.id.get_owner(), GameObjectOwner::Room);

		// должна быть загрузка объекта на текущий клиент
		let (_, create_command) = &room.test_out_commands[1];
		let (_, created_command) = &room.test_out_commands[0];
		assert!(matches!(create_command, S2CCommand::Create(ref command) if command.object_id
			.get_owner()==GameObjectOwner::Room));
		assert!(matches!(created_command, S2CCommand::Created(ref command) if command.object_id
			.get_owner()==GameObjectOwner::Room));
	}

	///
	/// Не должно быть двух объектов с владельцем Room с одним `singleton_key`
	///
	#[test]
	pub(crate) fn should_dont_create_more_one_object_with_one_singleton_key() {
		let (mut room, member_id, access_groups) = setup_one_player();

		let singleton_key = Some(BinaryValue::from([1, 2, 3].as_slice()));

		let member_object_id_1 = GameObjectId::new(100, GameObjectOwner::Member(member_id));
		let create_command = CreateGameObjectCommand {
			object_id: member_object_id_1,
			template: 777,
			access_groups,
		};
		create_command.execute(&mut room, member_id).unwrap();
		let created_command = C2SCreatedGameObjectCommand {
			object_id: member_object_id_1,
			room_owner: true,
			singleton_key: singleton_key.clone(),
		};
		created_command.execute(&mut room, member_id).unwrap();
		room.test_out_commands.clear();

		let member_object_id_2 = GameObjectId::new(101, GameObjectOwner::Member(member_id));
		let create_command = CreateGameObjectCommand {
			object_id: member_object_id_2,
			template: 777,
			access_groups,
		};
		create_command.execute(&mut room, member_id).unwrap();
		let created_command = C2SCreatedGameObjectCommand {
			object_id: member_object_id_2,
			room_owner: true,
			singleton_key,
		};
		created_command.execute(&mut room, member_id).unwrap();
		assert_eq!(room.objects.len(), 1);
		assert_eq!(room.test_out_commands.len(), 0);
	}
}
