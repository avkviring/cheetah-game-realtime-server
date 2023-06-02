use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::template::config::Permission;
use crate::room::Room;
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::float::{IncrementDoubleC2SCommand, SetDoubleCommand};
use cheetah_common::room::field::{Field, FieldType};
use cheetah_protocol::RoomMemberId;

impl ServerCommandExecutor for SetDoubleCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id;

		let action = |object: &mut GameObject| {
			object.doubles.set(self.field_id, self.value.clone());
			Ok(Some(S2CCommand::SetDouble(self.clone())))
		};

		room.send_command_from_action(
			object_id,
			Field {
				id: field_id,
				field_type: FieldType::Double,
			},
			member_id,
			Permission::Rw,
			None,
			action,
		)
	}
}

impl ServerCommandExecutor for IncrementDoubleC2SCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id;

		let action = |object: &mut GameObject| {
			let value = object.doubles.get(field_id).cloned().unwrap_or_default() + self.increment;
			object.doubles.set(field_id, value);
			Ok(Some(S2CCommand::SetDouble(SetDoubleCommand {
				object_id: self.object_id,
				field_id,
				value,
			})))
		};

		room.send_command_from_action(
			object_id,
			Field {
				id: field_id,
				field_type: FieldType::Double,
			},
			member_id,
			Permission::Rw,
			None,
			action,
		)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::float::{IncrementDoubleC2SCommand, SetDoubleCommand};
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::room::command::tests::setup_one_player;
	use crate::room::command::ServerCommandExecutor;

	#[test]
	fn should_set_double_command() {
		let (mut room, member_id, access_groups) = setup_one_player();
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups);
		let object_id = object.id;
		object.created = true;
		room.test_out_commands.clear();
		let command = SetDoubleCommand {
			object_id,
			field_id: 10,
			value: 100.100.into(),
		};
		command.execute(&mut room, member_id).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		assert_eq!(*object.doubles.get(10).unwrap(), 100.100);
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetDouble(c))) 
			if c==command));
	}

	#[test]
	fn should_increment_double_command() {
		let (mut room, member_id, access_groups) = setup_one_player();

		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups);
		object.created = true;
		let object_id = object.id;
		room.test_out_commands.clear();
		let command = IncrementDoubleC2SCommand {
			object_id,
			field_id: 10,
			increment: 100.100,
		};
		command.clone().execute(&mut room, member_id).unwrap();
		command.execute(&mut room, member_id).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		assert_eq!(*object.doubles.get(10).unwrap(), 200.200);

		let result = SetDoubleCommand {
			object_id,
			field_id: 10,
			value: 200.200,
		};

		room.test_out_commands.pop_back();
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetDouble(c))) 
			if c==result));
	}
}
