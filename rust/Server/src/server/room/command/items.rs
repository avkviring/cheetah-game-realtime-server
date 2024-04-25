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

		let item_config = object.config.get_items_config(&item.field_id);
		if deque.len() >= item_config.capacity {
			deque.pop_front();
		}
		deque.push_back(item.value.clone());
		Ok(Some(S2CCommand::AddItem(item.clone())))
	};

	room.send_command_from_action(object_id, member_id, None, action)
}

#[cfg(test)]
mod tests {
	use crate::server::room::command::items::add;
	use crate::server::room::config::member::MemberCreateParams;
	use crate::server::room::config::object::{GameObjectConfig, ItemConfig};
	use crate::server::room::config::room::RoomCreateParams;
	use crate::server::room::Room;
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::structure::BinaryField;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::buffer::Buffer;
	use cheetah_common::room::object::{GameObjectId, GameObjectTemplateId};
	use cheetah_common::room::owner::GameObjectOwner;
	use cheetah_game_realtime_protocol::RoomMemberId;
	use std::collections::VecDeque;

	#[test]
	pub(crate) fn should_add_items() {
		let (mut room, member_id, object_id) = setup(Default::default(), Default::default());
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
		assert_eq!(*structures, VecDeque::from([command_1.value.clone(), command_2.value.clone()]));
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::AddItem(c))) if c ==
			command_1.into()));
	}
	#[test]
	pub(crate) fn should_capacity_items() {
		let template = 10;
		let field_id = 5;
		let room_create_params = RoomCreateParams {
			name: "".to_string(),
			objects: vec![],
			configs: vec![(
				template,
				GameObjectConfig {
					items_config: vec![(field_id, ItemConfig { capacity: 1 })].into_iter().collect(),
				},
			)]
			.into_iter()
			.collect(),
		};

		let (mut room, member_id, object_id) = setup(room_create_params, template);
		let command_1 = BinaryField {
			object_id,
			field_id,
			value: Buffer::from(vec![1, 2, 3].as_slice()),
		};
		let command_2 = BinaryField {
			object_id,
			field_id,
			value: Buffer::from(vec![4, 5, 6].as_slice()),
		};
		add(&command_1, &mut room, member_id).unwrap();
		add(&command_2, &mut room, member_id).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		let structures = object.structures_fields.get(field_id).unwrap();
		assert_eq!(*structures, VecDeque::from([command_2.value]));
	}

	fn setup(room_create_params: RoomCreateParams, template_id: GameObjectTemplateId) -> (Room, RoomMemberId, GameObjectId) {
		let mut room = Room::new(0, room_create_params);
		let access_groups = AccessGroups(10);
		let member_id = room.register_member(MemberCreateParams::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups, template_id);
		object.created = true;
		let object_id = object.id;
		room.test_out_commands.clear();
		(room, member_id, object_id)
	}
}
