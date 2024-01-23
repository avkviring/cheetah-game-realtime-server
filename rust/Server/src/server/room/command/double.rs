use crate::server::room::command::ServerCommandError;
use crate::server::room::object::GameObject;
use crate::server::room::Room;
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::float::{DoubleField, IncrementDouble};
use cheetah_game_realtime_protocol::RoomMemberId;

pub(crate) fn set(command: &DoubleField, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let object_id = command.object_id;
	let action = |object: &mut GameObject| {
		object.double_fields.set(command.field_id, command.value);
		Ok(Some(S2CCommand::SetDouble(command.clone())))
	};
	room.send_command_from_action(object_id, member_id, None, action)
}

pub(crate) fn increment(increment: &IncrementDouble, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let field_id = increment.field_id;
	let object_id = increment.object_id;

	let action = |object: &mut GameObject| {
		let value = object.double_fields.get(field_id).cloned().unwrap_or_default() + increment.increment;
		object.double_fields.set(field_id, value);
		Ok(Some(S2CCommand::SetDouble(DoubleField {
			object_id: increment.object_id,
			field_id,
			value,
		})))
	};

	room.send_command_from_action(object_id, member_id, None, action)
}

#[cfg(test)]
mod tests {
	use crate::server::room::command::double::{increment, set};
	use crate::server::room::command::tests::setup_one_player;
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::float::{DoubleField, IncrementDouble};
	use cheetah_common::room::owner::GameObjectOwner;

	#[test]
	fn should_set_double_command() {
		let (mut room, member_id, access_groups) = setup_one_player();
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups, Default::default());
		let object_id = object.id;
		object.created = true;
		room.test_out_commands.clear();
		let command = DoubleField {
			object_id,
			field_id: 10,
			value: 100.100.into(),
		};
		set(&command, &mut room, member_id).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		assert_eq!(*object.double_fields.get(10).unwrap(), 100.100);
		assert!(matches!(room.test_out_commands.pop_back(), Some((..,S2CCommand::SetDouble(c))) if c==command));
	}

	#[test]
	fn should_increment_double_command() {
		let (mut room, member_id, access_groups) = setup_one_player();

		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups, Default::default());
		object.created = true;
		let object_id = object.id;
		room.test_out_commands.clear();
		let command = IncrementDouble {
			object_id,
			field_id: 10,
			increment: 100.100,
		};
		increment(&command, &mut room, member_id).unwrap();
		increment(&command, &mut room, member_id).unwrap();

		let object = room.get_object_mut(object_id).unwrap();
		assert_eq!(*object.double_fields.get(10).unwrap(), 200.200);

		let result = DoubleField {
			object_id,
			field_id: 10,
			value: 200.200,
		};

		room.test_out_commands.pop_back();
		assert!(matches!(room.test_out_commands.pop_back(), Some((.., S2CCommand::SetDouble(c))) 
			if c==result));
	}
}
