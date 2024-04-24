use cheetah_game_realtime_protocol::RoomMemberId;

use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::event::TargetEvent;
use cheetah_common::commands::types::structure::BinaryField;

use crate::server::room::command::ServerCommandError;
use crate::server::room::object::GameObject;
use crate::server::room::Room;

pub(crate) fn send(event: &BinaryField, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let object_id = event.object_id;
	let action = |_object: &mut GameObject| Ok(Some(S2CCommand::Event((*event).into())));
	room.send_command_from_action(object_id, member_id, None, action)
}

pub(crate) fn send_target(target_event: &TargetEvent, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let object_id = target_event.event.object_id;
	let target = target_event.target;
	let action = |_object: &mut GameObject| Ok(Some(S2CCommand::Event(target_event.event.clone().into())));
	room.send_command_from_action(object_id, member_id, Some(target), action)
}

#[cfg(test)]
mod tests {
	use crate::server::room::command::event::{send, send_target};
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::event::TargetEvent;
	use cheetah_common::commands::types::structure::BinaryField;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::buffer::Buffer;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::server::room::command::tests::setup_one_player;
	use crate::server::room::config::member::MemberCreateParams;
	use crate::server::room::config::room::RoomCreateParams;
	use crate::server::room::Room;

	#[test]
	pub(crate) fn should_send_event() {
		let (mut room, member_id, access_groups) = setup_one_player();
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups, Default::default());
		object.created = true;
		let object_id = object.id;
		room.test_out_commands.clear();

		let command = BinaryField {
			object_id,
			field_id: 100,
			value: Buffer::from(vec![1, 2, 3, 4, 5].as_slice()),
		};

		send(&command, &mut room, member_id).unwrap();
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::Event(c))) if c==command.into()));
	}

	#[test]
	pub(crate) fn should_send_event_to_member() {
		let template = RoomCreateParams::default();
		let access_groups = AccessGroups(10);

		let mut room = Room::new(0, template);
		let member1 = room.register_member(MemberCreateParams::stub(access_groups));
		let member2 = room.register_member(MemberCreateParams::stub(access_groups));
		let member3 = room.register_member(MemberCreateParams::stub(access_groups));

		room.mark_as_attached_in_test(member1).unwrap();
		room.mark_as_attached_in_test(member2).unwrap();
		room.mark_as_attached_in_test(member3).unwrap();

		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member1), access_groups, Default::default());
		object.created = true;
		let object_id = object.id;
		room.get_member_out_commands_for_test(member1).clear();
		room.get_member_out_commands_for_test(member2).clear();
		room.get_member_out_commands_for_test(member3).clear();

		let command = TargetEvent {
			target: member2,
			event: BinaryField {
				object_id,
				field_id: 100,
				value: Buffer::from(vec![1, 2, 3, 4, 5].as_slice()),
			},
		};

		send_target(&command, &mut room, member1).unwrap();
		assert!(matches!(room.get_member_out_commands_for_test(member1).pop_back(), None));
		assert!(matches!(room.get_member_out_commands_for_test(member2).pop_back(), Some(S2CCommand::Event(c)) if c.field_id == command.event.field_id));
		assert!(matches!(room.get_member_out_commands_for_test(member3).pop_back(), None));
	}
}
