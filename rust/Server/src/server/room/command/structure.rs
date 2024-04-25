use cheetah_game_realtime_protocol::RoomMemberId;

use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::structure::BinaryField;

use crate::server::room::command::ServerCommandError;
use crate::server::room::object::GameObject;
use crate::server::room::Room;

pub(crate) fn set(field: &BinaryField, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let object_id = field.object_id;
	let action = |object: &mut GameObject| {
		object.structure_fields.set(field.field_id, Box::new(field.value.clone()));
		Ok(Some(S2CCommand::SetStructure(field.clone().into())))
	};
	room.send_command_from_action(object_id, member_id, None, action)
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::structure::BinaryField;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::buffer::Buffer;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::server::room::command::structure;
	use crate::server::room::config::member::MemberCreateParams;
	use crate::server::room::config::room::RoomCreateParams;
	use crate::server::room::Room;

	#[test]
	pub(crate) fn should_set_structure() {
		let template = RoomCreateParams::default();
		let mut room = Room::new(0, template);
		let access_groups = AccessGroups(10);
		let member_id = room.register_member(MemberCreateParams::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups, Default::default());
		object.created = true;
		let object_id = object.id;

		room.test_out_commands.clear();
		let command = BinaryField {
			object_id,
			field_id: 100,
			value: Buffer::from(vec![1, 2, 3, 4, 5].as_slice()).into(),
		};

		structure::set(&command, &mut room, member_id).unwrap();
		let object = room.get_object_mut(object_id).unwrap();

		assert_eq!(**object.structure_fields.get(100).unwrap(), command.value);
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetStructure(c))) if c == 
			command.into()));
	}
}
