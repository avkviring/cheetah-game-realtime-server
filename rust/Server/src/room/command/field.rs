use cheetah_common::commands::field::Field;
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::field::DeleteFieldCommand;
use cheetah_common::commands::FieldType;
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
			match self.field_type {
				FieldType::Long => {
					object.longs.delete(self.field_id);
				}
				FieldType::Double => {
					object.doubles.delete(self.field_id);
				}
				FieldType::Structure => {
					object.structures.delete(self.field_id);
				}
				FieldType::Event => {}
			}

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
	use cheetah_common::commands::binary_value::Buffer;
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
		object.longs.set(1, Default::default());
		object.doubles.set(2, Default::default());
		object.structures.set(3, Default::default());

		let commands = [
			DeleteFieldCommand {
				object_id,
				field_id: 1,
				field_type: FieldType::Long,
			},
			DeleteFieldCommand {
				object_id,
				field_id: 2,
				field_type: FieldType::Double,
			},
			DeleteFieldCommand {
				object_id,
				field_id: 3,
				field_type: FieldType::Structure,
			},
		];
		commands.iter().cloned().for_each(|command| command.execute(&mut room, member_id).unwrap());

		let object = room.get_object_mut(object_id).unwrap();
		assert!(object.longs.get(1).is_none());
		assert!(object.doubles.get(2).is_none());
		assert!(object.structures.get(3).is_none());
		commands.iter().cloned().for_each(|command| {
			assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::DeleteField(c))) if c==command));
		});
	}

	#[test]
	pub(crate) fn should_delete_field() {
		let mut object = GameObject::new(GameObjectId::default(), 0, Default::default(), false);

		object.structures.set(1, Buffer::from([1, 2, 3].as_ref()));
		object.structures.delete(1);

		object.doubles.set(2, 10.0);
		object.doubles.delete(2);

		object.longs.set(3, 20);
		object.longs.delete(3);

		assert!(object.structures.get(1).is_none());
		assert!(object.doubles.get(2).is_none());
		assert!(object.longs.get(3).is_none());
	}
}
