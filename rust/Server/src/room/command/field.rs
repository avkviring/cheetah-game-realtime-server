use cheetah_common::commands::field::Field;
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::field::DeleteFieldCommand;
use cheetah_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::template::config::Permission;
use crate::room::Room;

impl ServerCommandExecutor for DeleteFieldCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let field_id = self.field_id;
		let object_id = self.object_id;
		let action = |object: &mut GameObject| {
			object.delete_field(self.field_id, self.field_type);
			Ok(Some(S2CCommand::DeleteField(self.clone())))
		};
		room.send_command_from_action(
			object_id,
			Field {
				id: field_id,
				field_type: self.field_type,
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
	use cheetah_common::commands::types::field::DeleteFieldCommand;
	use cheetah_common::commands::FieldType;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::room::command::tests::setup_one_player;
	use crate::room::command::ServerCommandExecutor;
	use crate::room::object::GameObject;

	#[test]
	fn should_command() {
		let (mut room, member_id, access_groups) = setup_one_player();
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups);
		let object_id = object.id;
		object.created = true;
		object.set_field(10, 100).unwrap();
		let command = DeleteFieldCommand {
			object_id,
			field_id: 10,
			field_type: FieldType::Long,
		};
		command.execute(&mut room, member_id).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		assert!(object.get_field::<i64>(10).is_none());
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::DeleteField(c))) if c==command));
	}

	#[test]
	pub(crate) fn should_delete_field() {
		let mut object = GameObject::new(GameObjectId::default(), 0, Default::default(), false);

		object.set_field(1, [1, 2, 3].as_ref()).unwrap();
		object.delete_field(1, FieldType::Structure);

		object.set_field(2, 10.0).unwrap();
		object.delete_field(2, FieldType::Double);

		object.set_field(3, 20).unwrap();
		object.delete_field(3, FieldType::Long);

		assert!(object.get_field::<Vec<u8>>(1).is_none());
		assert!(object.get_field::<f64>(2).is_none());
		assert!(object.get_field::<i64>(3).is_none());
	}
}
