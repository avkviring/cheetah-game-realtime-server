use cheetah_relay_common::commands::command::event::EventCommand;
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::ServerCommandExecutor;
use crate::room::Room;

impl ServerCommandExecutor for EventCommand {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object_mut(&self.object_id) {
			let groups = object.access_groups.clone();
			room.send_to_group(groups, S2CCommand::Event(self))
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::event::EventCommand;
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::RoomTemplate;
	use crate::room::tests::from_vec;
	use crate::room::Room;

	#[test]
	pub fn should_send_event() {
		let mut room = Room::new(RoomTemplate::default(), Default::default());
		let object_id = room.create_object(&0).id.clone();
		room.out_commands.clear();

		let command = EventCommand {
			object_id: object_id.clone(),
			field_id: 100,
			event: from_vec(vec![1, 2, 3, 4, 5]),
		};
		command.clone().execute(&mut room, &32);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::Event(c))) if c==command));
	}

	#[test]
	pub fn should_not_panic_when_missing_object() {
		let mut room = Room::new(RoomTemplate::default(), Default::default());
		let command = EventCommand {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 100,
			event: from_vec(vec![1, 2, 3, 4, 5]),
		};
		command.execute(&mut room, &32);
	}
}
