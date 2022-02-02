use cheetah_matches_relay_common::commands::types::load::CreatedGameObjectCommand;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::CreateCommandsCollector;
use crate::room::Room;

impl ServerCommandExecutor for CreatedGameObjectCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let room_id = room.id;
		let object = room.get_object_mut(&self.object_id)?;
		if !object.created {
			let groups = object.access_groups;
			object.created = true;
			// объект полностью загружен - теперь его надо загрузить остальным клиентам
			let mut commands = CreateCommandsCollector::new();
			object.collect_create_commands(&mut commands);
			let template = object.template_id;
			room.send_to_members(groups, template, commands.as_slice(), |user| user.id != user_id)
		} else {
			return Err(ServerCommandError::Error(format!(
				"room[({:?})] object ({:?}) already created",
				room_id, object.id
			)));
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::s2c::S2CCommand;
	use cheetah_matches_relay_common::commands::types::load::CreatedGameObjectCommand;

	use crate::room::command::tests::setup_two_players;
	use crate::room::command::{ServerCommandError, ServerCommandExecutor};

	///
	/// - Команда должна приводить к рассылки оповещения для пользователей
	/// - Команда не должна отсылаться обратно пользователю
	///
	#[test]
	pub fn should_send_commands() {
		let (mut room, object_id, user1, user2) = setup_two_players();
		room.test_mark_as_connected(user1).unwrap();
		room.test_mark_as_connected(user2).unwrap();
		let command = CreatedGameObjectCommand {
			object_id: object_id.clone(),
		};
		command.execute(&mut room, user1).unwrap();

		assert!(room.test_get_user_out_commands(user1).is_empty());
		assert!(matches!(
			room.test_get_user_out_commands(user2).get(0),
			Some(S2CCommand::Create(c)) if c.object_id == object_id
		));

		assert!(matches!(
			room.test_get_user_out_commands(user2).get(1),
			Some(S2CCommand::Created(c)) if c.object_id == object_id
		));
	}

	///
	/// Команда должна отметить объект как загруженный
	///
	#[test]
	pub fn should_switch_object_to_created_state() {
		let (mut room, object_id, user1, _) = setup_two_players();
		let command = CreatedGameObjectCommand {
			object_id: object_id.clone(),
		};
		room.out_commands.clear();
		command.execute(&mut room, user1).unwrap();

		let object = room.get_object_mut(&object_id).unwrap();
		assert!(object.created);
	}

	///
	/// Не рассылать команду если объект уже создан
	/// Такого быть не должно, однако проверить стоит, так как команду могут послать умышленно.
	///
	#[test]
	pub fn should_dont_send_command_if_object_already_created() {
		let (mut room, object_id, user1, _) = setup_two_players();
		let object = room.get_object_mut(&object_id).unwrap();
		object.created = true;
		let command = CreatedGameObjectCommand {
			object_id: object_id.clone(),
		};
		room.out_commands.clear();

		assert!(matches!(command.execute(&mut room, user1), Err(ServerCommandError::Error(_))));
		assert!(matches!(room.out_commands.pop_back(), None));
	}
}
