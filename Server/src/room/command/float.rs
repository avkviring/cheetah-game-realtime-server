use cheetah_relay_common::commands::command::float_counter::{IncrementFloat64C2SCommand, SetFloat64Command};
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::hash::UserPublicKey;

use crate::room::{Room, User};
use crate::room::command::ServerCommandExecutor;

impl ServerCommandExecutor for IncrementFloat64C2SCommand {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			let value = object.fields.floats
				.entry(self.field_id)
				.and_modify(|v| *v += self.increment)
				.or_insert(self.increment)
				.clone();
			
			let access_groups = object.access_groups.clone();
			room.send(access_groups, S2CCommandUnion::SetFloat64(
				SetFloat64Command {
					object_id: self.object_id,
					field_id: self.field_id,
					value,
				}),
			);
		}
	}
}


impl ServerCommandExecutor for SetFloat64Command {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			object.fields.floats.insert(self.field_id, self.value);
			let access_groups = object.access_groups;
			room.send(access_groups, S2CCommandUnion::SetFloat64(self));
		}
	}
}


#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::float_counter::SetFloat64Command;
	use cheetah_relay_common::commands::command::S2CCommandUnion::SetFloat64;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ClientOwner;
	
	use crate::room::{MockRoom, Room, User};
	use crate::room::command::ServerCommandExecutor;
	
	#[test]
	fn test() {
		// let mut room = MockRoom::new(0);
		// let command = SetFloat64Command {
		// 	object_id: GameObjectId::new(0, ClientOwner::Client(12)),
		// 	field_id: 10,
		// 	value: 100.100,
		// };
		// command.execute(&mut room,&12);
	}
}