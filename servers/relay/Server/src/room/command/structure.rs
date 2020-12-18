use cheetah_relay_common::commands::command::structure::StructureCommand;
use cheetah_relay_common::commands::command::{HeaplessBuffer, S2CCommand};
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::ServerCommandExecutor;
use crate::room::object::GameObject;
use crate::room::template::config::Permission;
use crate::room::types::FieldType;
use crate::room::Room;

impl ServerCommandExecutor for StructureCommand {
	fn execute(self, room: &mut Room, user_public_key: &UserPublicKey) {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();
		let action = |object: &mut GameObject| {
			object.structures.insert(self.field_id, self.structure.to_vec());
			Option::Some(S2CCommand::SetStruct(self))
		};
		room.do_command(&object_id, &field_id, FieldType::Structure, user_public_key, Permission::Rw, action);
	}
}

impl GameObject {
	pub fn structures_to_commands(&self, commands: &mut Vec<S2CCommand>) {
		self.structures.iter().for_each(|(k, v)| {
			let structure = HeaplessBuffer::from_slice(&v.as_slice()).unwrap();
			commands.push(S2CCommand::SetStruct(StructureCommand {
				object_id: self.id.clone(),
				field_id: k.clone(),
				structure,
			}));
		})
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::structure::StructureCommand;
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::tests::from_vec;
	use crate::room::Room;

	#[test]
	pub fn should_set_structure() {
		let mut room = Room::default();
		let object_id = room.create_object(&0).id.clone();
		room.out_commands.clear();
		let command = StructureCommand {
			object_id: object_id.clone(),
			field_id: 100,
			structure: from_vec(vec![1, 2, 3, 4, 5]),
		};

		command.clone().execute(&mut room, &32);
		let object = room.get_object_mut(&object_id).unwrap();

		assert_eq!(*object.structures.get(&100).unwrap(), command.structure.to_vec());
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetStruct(c))) if c==command));
	}

	#[test]
	pub fn should_not_panic_when_missing_object() {
		let mut room = Room::default();
		let command = StructureCommand {
			object_id: GameObjectId::new(10, ObjectOwner::Root),
			field_id: 100,
			structure: from_vec(vec![1, 2, 3, 4, 5]),
		};
		command.execute(&mut room, &32);
	}
}
