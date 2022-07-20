use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::field::SetFieldCommand;
use cheetah_matches_relay_common::commands::types::float::IncrementDoubleC2SCommand;
use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::{Field, GameObject};
use crate::room::template::config::Permission;
use crate::room::Room;

impl ServerCommandExecutor for IncrementDoubleC2SCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();

		let action = |object: &mut GameObject| {
			let value = if let Some(value) = object.get_field(field_id) {
				let new_value = value + self.increment;
				object.set_field(field_id, new_value)?;
				new_value
			} else {
				object.set_field(field_id, self.increment)?;
				self.increment
			};
			Ok(Some(S2CCommand::SetField(SetFieldCommand {
				object_id: self.object_id.clone(),
				field_id,
				value: value.into(),
			})))
		};

		room.send_command_from_action(
			&object_id,
			Field {
				id: field_id,
				field_type: FieldType::Double,
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::s2c::S2CCommand;
	use cheetah_matches_relay_common::commands::types::field::SetFieldCommand;
	use cheetah_matches_relay_common::commands::types::float::IncrementDoubleC2SCommand;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;

	use crate::room::command::tests::setup_one_player;
	use crate::room::command::ServerCommandExecutor;

	#[test]
	fn should_set_double_command() {
		let (mut room, user, access_groups) = setup_one_player();
		let object = room.test_create_object_with_not_created_state(
			GameObjectOwner::Member(user),
			access_groups,
		);
		let object_id = object.id.clone();
		object.created = true;
		room.test_out_commands.clear();
		let command = SetFieldCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100.100.into(),
		};
		command.execute(&mut room, user).unwrap();

		let object = room.get_object(&object_id).unwrap();
		assert_eq!(*object.get_field::<f64>(10).unwrap() as u64, 100);
		assert!(
			matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetField(c))) if c==command)
		);
	}

	#[test]
	fn should_increment_double_command() {
		let (mut room, user, access_groups) = setup_one_player();

		let object = room.test_create_object_with_not_created_state(
			GameObjectOwner::Member(user),
			access_groups,
		);
		object.created = true;
		let object_id = object.id.clone();
		room.test_out_commands.clear();
		let command = IncrementDoubleC2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: 100.100,
		};
		command.clone().execute(&mut room, user).unwrap();
		command.execute(&mut room, user).unwrap();

		let object = room.get_object(&object_id).unwrap();
		assert_eq!(*object.get_field::<f64>(10).unwrap() as u64, 200);

		let result = SetFieldCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 200.200.into(),
		};

		room.test_out_commands.pop_back();
		assert!(
			matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetField(c))) if c==result)
		);
	}
}
