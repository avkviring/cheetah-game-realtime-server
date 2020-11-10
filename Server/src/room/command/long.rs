use cheetah_relay_common::commands::command::long_counter::{IncrementLongC2SCommand, SetLongCommand};
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::ServerCommandExecutor;
use crate::room::Room;

impl ServerCommandExecutor for IncrementLongC2SCommand {
	fn execute(self, room: &mut dyn Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			let value = object.fields.longs
				.entry(self.field_id)
				.and_modify(|v| *v += self.increment)
				.or_insert(self.increment)
				.clone();
			
			let access_groups = object.access_groups.clone();
			room.send_to_group(access_groups, S2CCommandUnion::SetLong(
				SetLongCommand {
					object_id: self.object_id,
					field_id: self.field_id,
					value,
				}),
			);
		}
	}
}


impl ServerCommandExecutor for SetLongCommand {
	fn execute(self, room: &mut dyn Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			object.fields.longs.insert(self.field_id, self.value);
			let access_groups = object.access_groups.clone();
			room.send_to_group(access_groups, S2CCommandUnion::SetLong(self));
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::long_counter::{IncrementLongC2SCommand, SetLongCommand};
	use cheetah_relay_common::commands::command::S2CCommandUnion;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ClientOwner;
	
	use crate::room::command::ServerCommandExecutor;
	use crate::room::Room;
	use crate::room::tests::RoomStub;
	
	#[test]
	fn should_set_long_command() {
		let mut room = RoomStub::new();
		let object_id = room.create_object(&0).id.clone();
		let command = SetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100,
		};
		command.clone().execute(&mut room, &12);
		
		let object = room.get_object(&object_id).unwrap();
		assert_eq!(*object.fields.longs.get(&10).unwrap(), 100);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommandUnion::SetLong(c))) if c==command));
	}
	
	#[test]
	fn should_increment_long_command() {
		let mut room = RoomStub::new();
		let object_id = room.create_object(&0).id.clone();
		let command = IncrementLongC2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: 100,
		};
		command.clone().execute(&mut room, &12);
		command.clone().execute(&mut room, &12);
		
		let object = room.get_object(&object_id).unwrap();
		assert_eq!(*object.fields.longs.get(&10).unwrap(), 200);
		
		let result = SetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 200,
		};
		room.out_commands.pop_back();
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommandUnion::SetLong(c))) if c==result));
	}
	
	#[test]
	fn should_not_panic_when_set_long_command_not_panic_for_missing_object() {
		let mut room = RoomStub::new();
		let command = SetLongCommand {
			object_id: GameObjectId::new(10, ClientOwner::Root),
			field_id: 10,
			value: 100,
		};
		command.execute(&mut room, &12);
	}
	
	#[test]
	fn should_not_panic_when_increment_float_command_not_panic_for_missing_object() {
		let mut room = RoomStub::new();
		let command = IncrementLongC2SCommand {
			object_id: GameObjectId::new(10, ClientOwner::Root),
			field_id: 10,
			increment: 100,
		};
		command.execute(&mut room, &12);
	}
}
