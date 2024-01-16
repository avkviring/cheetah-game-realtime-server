use cheetah_game_realtime_protocol::RoomMemberId;

use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::structure::BinaryField;

use crate::server::room::command::ServerCommandError;
use crate::server::room::object::fields::vec::Items;
use crate::server::room::object::GameObject;
use crate::server::room::Room;

pub(crate) fn add(item: &BinaryField, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let object_id = item.object_id;

	let action = |object: &mut GameObject| {
		let deque = match object.structures_fields.get_mut(item.field_id) {
			None => {
				let deque: Items = Default::default();
				object.structures_fields.set(item.field_id, deque);
				object.structures_fields.get_mut(item.field_id).unwrap()
			}
			Some(deque) => deque,
		};
		if deque.len() > 50 {
			deque.pop_front();
		}
		deque.push_back(item.value);
		Ok(Some(S2CCommand::AddItem((*item).into())))
	};

	room.send_command_from_action(object_id, member_id, None, action)
}

#[cfg(test)]
mod tests {
	use crate::server::room::command::items::add;
	use crate::server::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::server::room::Room;
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::structure::BinaryField;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::buffer::Buffer;
	use cheetah_common::room::owner::GameObjectOwner;
	use std::collections::VecDeque;

	#[test]
	pub(crate) fn should_add_structure() {
		let template = RoomTemplate::default();
		let mut room = Room::from_template(template);
		let access_groups = AccessGroups(10);
		let member_id = room.register_member(MemberTemplate::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups);
		object.created = true;
		let object_id = object.id;

		room.test_out_commands.clear();

		let command_1 = BinaryField {
			object_id,
			field_id: 100,
			value: Buffer::from(vec![1, 2, 3].as_slice()),
		};

		let command_2 = BinaryField {
			object_id,
			field_id: 100,
			value: Buffer::from(vec![4, 5, 6].as_slice()),
		};

		add(&command_1, &mut room, member_id).unwrap();
		add(&command_2, &mut room, member_id).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		let structures = object.structures_fields.get(100).unwrap();
		assert_eq!(*structures, VecDeque::from([command_1.value, command_2.value]));
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::AddItem(c))) if c ==
			command_1.into()));
	}
}
