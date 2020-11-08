use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::command::structure::StructureCommand;
use cheetah_relay_common::constants::FieldID;
use crate::room::command::ServerCommandExecutor;
use crate::room::{Room, User};
use cheetah_relay_common::commands::hash::UserPublicKey;


impl ServerCommandExecutor for StructureCommand {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			object.fields.structures.insert(self.field_id, self.structure.clone());
			let groups = object.access_groups.clone();
			room.send(groups, S2CCommandUnion::SetStruct(self))
		}
	}
}
