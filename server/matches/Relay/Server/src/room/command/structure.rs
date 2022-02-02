use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::structure::SetStructureCommand;
use cheetah_matches_relay_common::commands::{CommandBuffer, FieldType};
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::{CreateCommandsCollector, Field, GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::Permission;
use crate::room::Room;

impl ServerCommandExecutor for SetStructureCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();
		let action = |object: &mut GameObject| {
			object.set_structure(self.field_id, self.structure.as_slice())?;
			Ok(Some(S2CCommand::SetStructure(self.clone())))
		};
		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id,
				field_type: FieldType::Structure,
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		)
	}
}

impl GameObject {
	pub fn structures_to_commands(&self, commands: &mut CreateCommandsCollector) -> Result<(), S2CommandWithFieldInfo> {
		for (field_id, v) in self.get_structures().iter() {
			let structure = CommandBuffer::from_slice(v.as_slice()).unwrap();
			let command = S2CommandWithFieldInfo {
				field: Option::Some(Field {
					id: *field_id,
					field_type: FieldType::Structure,
				}),
				command: S2CCommand::SetStructure(SetStructureCommand {
					object_id: self.id.clone(),
					field_id: *field_id,
					structure,
				}),
			};
			commands.push(command)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::s2c::S2CCommand;
	use cheetah_matches_relay_common::commands::types::structure::SetStructureCommand;
	use cheetah_matches_relay_common::room::access::AccessGroups;

	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::{RoomTemplate, UserTemplate};
	use crate::room::tests::from_vec;
	use crate::room::Room;

	#[test]
	pub fn should_set_structure() {
		let template = RoomTemplate::default();
		let mut room = Room::from_template(template);
		let access_groups = AccessGroups(10);
		let user = room.register_user(UserTemplate::stub(access_groups));
		let object = room.create_object(user, access_groups);
		object.created = true;
		let object_id = object.id.clone();

		room.out_commands.clear();
		let command = SetStructureCommand {
			object_id: object_id.clone(),
			field_id: 100,
			structure: from_vec(vec![1, 2, 3, 4, 5]),
		};

		command.execute(&mut room, user).unwrap();
		let object = room.get_object_mut(&object_id).unwrap();

		assert_eq!(*object.get_structure(&100).unwrap(), command.structure.to_vec());
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetStructure(c))) if c==command));
	}
}
