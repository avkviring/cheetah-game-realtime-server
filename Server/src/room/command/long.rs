use crate::room::command::ServerCommandExecutor;
use cheetah_relay_common::commands::command::long_counter::{IncrementLongC2SCommand, SetLongCommand};
use crate::room::Room;
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::hash::UserPublicKey;

impl ServerCommandExecutor for IncrementLongC2SCommand {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			let value = object.fields.longs
				.entry(self.field_id)
				.and_modify(|v| *v += self.increment)
				.or_insert(self.increment)
				.clone();
			
			let access_groups = object.access_groups.clone();
			room.send(access_groups, S2CCommandUnion::SetLong(
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
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			object.fields.longs.insert(self.field_id, self.value);
			let access_groups = object.access_groups.clone();
			room.send(access_groups, S2CCommandUnion::SetLong(self));
		}
	}
}


