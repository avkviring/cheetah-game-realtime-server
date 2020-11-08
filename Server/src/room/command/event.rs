use cheetah_relay_common::commands::command::event::EventCommand;
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::hash::UserPublicKey;

use crate::room::{Room, User};
use crate::room::command::ServerCommandExecutor;

impl ServerCommandExecutor for EventCommand {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			let groups = object.access_groups.clone();
			room.send(groups, S2CCommandUnion::Event(self))
		}
	}
}