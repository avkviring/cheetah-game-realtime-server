use cheetah_relay_common::commands::command::load::CreatedGameObjectCommand;
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::ServerCommandExecutor;
use crate::room::Room;

impl ServerCommandExecutor for CreatedGameObjectCommand {
	fn execute(self, room: &mut Room, _user_public_key: &UserPublicKey) {
		let room_id = room.id;
		if let Some(object) = room.get_object_mut(&self.object_id) {
			if !object.created {
				let groups = object.access_groups.clone();
				object.created = true;
				room.send_to_group(groups, S2CCommand::Created(self), |_| true)
			} else {
				log::error!("room[({:?})] object ({:?}) already created", room_id, object.id);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::load::CreatedGameObjectCommand;
	use cheetah_relay_common::commands::command::S2CCommand;

	use crate::room::command::tests::setup;
	use crate::room::command::ServerCommandExecutor;

	///
	/// Команда должна приводить к рассылки оповещения для пользователей
	///
	#[test]
	pub fn should_send_commands() {
		let (mut room, object_id, user1, _) = setup();
		let command = CreatedGameObjectCommand {
			object_id: object_id.clone(),
		};
		room.out_commands.clear();
		command.execute(&mut room, &user1);

		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::Created(c))) if c.object_id==object_id));
	}

	///
	/// Команда должна отметить объект как загруженный
	///
	#[test]
	pub fn should_switch_object_to_created_state() {
		let (mut room, object_id, user1, _) = setup();
		let command = CreatedGameObjectCommand {
			object_id: object_id.clone(),
		};
		room.out_commands.clear();
		command.execute(&mut room, &user1);

		let object = room.get_object_mut(&object_id).unwrap();
		assert!(object.created);
	}

	///
	/// Не рассылать команду если объект уже создан
	/// Такого быть не должно, однако проверить стоит, так как команду могут послать умышленно.
	///
	#[test]
	pub fn should_dont_send_command_if_object_already_created() {
		let (mut room, object_id, user1, _) = setup();
		let object = room.get_object_mut(&object_id).unwrap();
		object.created = true;
		let command = CreatedGameObjectCommand {
			object_id: object_id.clone(),
		};
		room.out_commands.clear();
		command.execute(&mut room, &user1);

		assert!(matches!(room.out_commands.pop_back(), None));
	}
}
