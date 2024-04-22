use cheetah_game_realtime_protocol::RoomMemberId;

use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::field::DeleteField;
use cheetah_common::room::field::FieldType;

use crate::server::room::command::ServerCommandError;
use crate::server::room::object::GameObject;
use crate::server::room::Room;

pub(crate) fn delete(field: &DeleteField, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let object_id = field.object_id;
	let action = |object: &mut GameObject| {
		match field.field_type {
			FieldType::Long => {
				object.long_fields.delete(field.field_id);
			}
			FieldType::Double => {
				object.double_fields.delete(field.field_id);
			}
			FieldType::Structure => {
				object.structure_fields.delete(field.field_id);
			}
			FieldType::Items => {
				object.structures_fields.delete(field.field_id);
			}
			FieldType::Event => {}
		}

		Ok(Some(S2CCommand::DeleteField(field.clone())))
	};
	room.send_command_from_action(object_id, member_id, None, action)
}

#[cfg(test)]
mod tests {
	use crate::server::room::command::field::delete;
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::field::DeleteField;
	use cheetah_common::room::buffer::Buffer;
	use cheetah_common::room::field::FieldType;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::server::room::command::tests::setup_one_player;
	use crate::server::room::object::GameObject;

	#[test]
	fn should_command() {
		let (mut room, member_id, access_groups) = setup_one_player();
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups, Default::default());
		let object_id = object.id;
		object.created = true;
		object.long_fields.set(1, Default::default());
		object.double_fields.set(2, Default::default());
		object.structure_fields.set(3, Default::default());

		let commands = [
			DeleteField {
				object_id,
				field_id: 1,
				field_type: FieldType::Long,
			},
			DeleteField {
				object_id,
				field_id: 2,
				field_type: FieldType::Double,
			},
			DeleteField {
				object_id,
				field_id: 3,
				field_type: FieldType::Structure,
			},
		];
		commands.iter().cloned().for_each(|command| delete(&command, &mut room, member_id).unwrap());

		let object = room.get_object_mut(object_id).unwrap();
		assert!(object.long_fields.get(1).is_none());
		assert!(object.double_fields.get(2).is_none());
		assert!(object.structure_fields.get(3).is_none());
		commands.iter().cloned().for_each(|command| {
			assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::DeleteField(c))) if c==command));
		});
	}

	#[test]
	pub(crate) fn should_delete_field() {
		let mut object = GameObject::new(GameObjectId::default(), 0, Default::default(), Default::default(), false);

		object.structure_fields.set(1, Box::new(Buffer::from([1, 2, 3].as_ref())));
		object.structure_fields.delete(1);

		object.double_fields.set(2, 10.0);
		object.double_fields.delete(2);

		object.long_fields.set(3, 20);
		object.long_fields.delete(3);

		assert!(object.structure_fields.get(1).is_none());
		assert!(object.double_fields.get(2).is_none());
		assert!(object.long_fields.get(3).is_none());
	}
}
