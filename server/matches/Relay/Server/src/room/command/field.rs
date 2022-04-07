use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::field::DeleteFieldCommand;
use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::{Field, GameObject};
use crate::room::template::config::Permission;
use crate::room::Room;

impl ServerCommandExecutor for DeleteFieldCommand {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id.clone();
		let action = |object: &mut GameObject| {
			object.delete_field(self.field_type, self.field_id);
			Ok(Some(S2CCommand::DeleteField(self.clone())))
		};
		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id,
				field_type: self.field_type,
			},
			user_id,
			Permission::Rw,
			Option::None,
			action,
		)
	}
}

impl GameObject {
	pub fn delete_field(&mut self, field_type: FieldType, field_id: FieldId) {
		match field_type {
			FieldType::Long => {
				self.longs.remove(&field_id);
			}
			FieldType::Double => {
				self.doubles.remove(&field_id);
			}
			FieldType::Structure => {
				self.structures.remove(&field_id);
			}
			FieldType::Event => {}
		};
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::s2c::S2CCommand;
	use cheetah_matches_relay_common::commands::types::field::DeleteFieldCommand;
	use cheetah_matches_relay_common::commands::FieldType;
	use cheetah_matches_relay_common::room::object::GameObjectId;

	use crate::room::command::tests::setup_one_player;
	use crate::room::command::ServerCommandExecutor;
	use crate::room::object::GameObject;

	#[test]
	fn should_command() {
		let (mut room, user, access_groups) = setup_one_player();
		let object = room.test_create_object(user, access_groups);
		let object_id = object.id.clone();
		object.created = true;
		object.set_long(10, 100).unwrap();
		let command = DeleteFieldCommand {
			object_id: object_id.clone(),
			field_id: 10,
			field_type: FieldType::Long,
		};
		command.execute(&mut room, user).unwrap();

		let object = room.get_object_mut(&object_id).unwrap();
		assert!(object.get_long(&10).is_none());
		assert!(matches!(room.out_commands.pop_back(), Some((.., S2CCommand::DeleteField(c))) if c==command));
	}

	#[test]
	pub fn should_delete_field() {
		let mut object = GameObject::new(GameObjectId::default(), 0, Default::default(), false);

		object.set_structure(1, &[1, 2, 3]).unwrap();
		object.delete_field(FieldType::Structure, 1);

		object.set_double(2, 10.0).unwrap();
		object.delete_field(FieldType::Double, 2);

		object.set_long(3, 20).unwrap();
		object.delete_field(FieldType::Long, 3);

		assert!(object.get_structure(&1).is_none());
		assert!(object.get_double(&2).is_none());
		assert!(object.get_long(&3).is_none());
	}
}
