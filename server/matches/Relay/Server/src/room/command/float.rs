use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::float::{IncrementDoubleC2SCommand, SetDoubleCommand};
use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::command::ServerCommandExecutor;
use crate::room::object::{CreateCommandsCollector, Field, GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::Permission;
use crate::room::Room;

impl ServerCommandExecutor for IncrementDoubleC2SCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();

		let action = |object: &mut GameObject| {
			let value = if let Some(value) = object.get_float(&field_id) {
				let new_value = value + self.increment;
				object.set_float(field_id, new_value);
				new_value
			} else {
				object.set_float(field_id, self.increment);
				self.increment
			};
			Option::Some(S2CCommand::SetDouble(SetDoubleCommand {
				object_id: self.object_id.clone(),
				field_id,
				value,
			}))
		};

		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id,
				field_type: FieldType::Double,
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		);
	}
}

impl ServerCommandExecutor for SetDoubleCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();

		let action = |object: &mut GameObject| {
			object.set_float(self.field_id, self.value);
			Option::Some(S2CCommand::SetDouble(self.clone()))
		};
		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id,
				field_type: FieldType::Double,
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		);
	}
}

impl GameObject {
	pub fn floats_to_commands(&self, commands: &mut CreateCommandsCollector) -> Result<(), S2CommandWithFieldInfo> {
		for (field_id, v) in self.get_floats() {
			let command = S2CommandWithFieldInfo {
				field: Option::Some(Field {
					id: *field_id,
					field_type: FieldType::Double,
				}),
				command: S2CCommand::SetDouble(SetDoubleCommand {
					object_id: self.id.clone(),
					field_id: *field_id,
					value: *v,
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
	use cheetah_matches_relay_common::commands::types::float::{IncrementDoubleC2SCommand, SetDoubleCommand};
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;

	use crate::room::command::tests::setup_one_player;
	use crate::room::command::ServerCommandExecutor;

	#[test]
	fn should_set_float_command() {
		let (mut room, user, access_groups) = setup_one_player();
		let object = room.create_object(user, access_groups);
		let object_id = object.id.clone();
		object.created = true;
		room.out_commands.clear();
		let command = SetDoubleCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 100.100,
		};
		command.clone().execute(&mut room, user);

		let object = room.get_object_mut(&object_id).unwrap();
		assert_eq!(*object.get_float(&10).unwrap() as u64, 100);
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetDouble(c))) if c==command));
	}

	#[test]
	fn should_increment_float_command() {
		let (mut room, user, access_groups) = setup_one_player();

		let object = room.create_object(user, access_groups);
		object.created = true;
		let object_id = object.id.clone();
		room.out_commands.clear();
		let command = IncrementDoubleC2SCommand {
			object_id: object_id.clone(),
			field_id: 10,
			increment: 100.100,
		};
		command.clone().execute(&mut room, user);
		command.execute(&mut room, user);

		let object = room.get_object_mut(&object_id).unwrap();
		assert_eq!(*object.get_float(&10).unwrap() as u64, 200);

		let result = SetDoubleCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 200.200,
		};

		room.out_commands.pop_back();
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::SetDouble(c))) if c==result));
	}

	#[test]
	fn should_not_panic_when_increment_float_command_not_panic_for_missing_object() {
		let (mut room, user, _) = setup_one_player();

		let command = IncrementDoubleC2SCommand {
			object_id: GameObjectId::new(10, GameObjectOwner::Room),
			field_id: 10,
			increment: 100.100,
		};
		command.execute(&mut room, user);
	}
}
