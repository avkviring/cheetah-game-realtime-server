use cheetah_relay_common::commands::command::event::EventCommand;
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::hash::UserPublicKey;

use crate::room::{Room, User};
use crate::room::command::ServerCommandExecutor;

impl ServerCommandExecutor for EventCommand {
	fn execute(self, room: &mut dyn Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			let groups = object.access_groups.clone();
			room.send(groups, S2CCommandUnion::Event(self))
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::event::EventCommand;
	use cheetah_relay_common::commands::command::S2CCommandUnion;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ClientOwner;
	
	use crate::room::command::ServerCommandExecutor;
	use crate::room::tests::RoomStub;
	
	#[test]
	pub fn should_send_event() {
		let mut room = RoomStub::new();
		let object_id = room.create_object();
		let command = EventCommand {
			object_id: object_id.clone(),
			field_id: 100,
			event: vec![1, 2, 3, 4, 5],
		};
		command.clone().execute(&mut room, &32);
		assert!(matches!(room.out_command.pop_back(), Some((.., S2CCommandUnion::Event(c))) if c==command));
	}
	
	#[test]
	pub fn should_not_panic_when_missing_object() {
		let mut room = RoomStub::new();
		let command = EventCommand {
			object_id: GameObjectId::new(10, ClientOwner::Root),
			field_id: 100,
			event: vec![1, 2, 3, 4, 5],
		};
		command.execute(&mut room, &32);
	}
}