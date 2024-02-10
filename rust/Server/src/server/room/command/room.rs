use crate::server::room::command::ServerCommandError;
use crate::server::room::object::S2CCommandsCollector;
use crate::server::room::Room;
use cheetah_common::room::object::GameObjectTemplateId;
use cheetah_game_realtime_protocol::RoomMemberId;

pub fn attach_to_room(room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let member = room.get_member_mut(&member_id)?;
	member.attached = true;
	let access_group = member.template.groups;
	let mut command_collector = Vec::<(GameObjectTemplateId, S2CCommandsCollector)>::new();
	room.objects
		.iter_mut()
		.filter(|(_, o)| o.created)
		.filter(|(_, o)| o.access_groups.contains_any(&access_group))
		.map(|(_, o)| {
			let mut commands = S2CCommandsCollector::new();
			o.collect_create_commands(&mut commands);
			(o.template_id, commands)
		})
		.for_each(|v| command_collector.push(v));
	for (_template, commands) in command_collector.iter() {
		room.send_to_member(&member_id, commands.as_slice())?;
	}
	Ok(())
}

pub fn detach_from_room(room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let member = room.get_member_mut(&member_id)?;
	member.attached = false;
	Ok(())
}

#[cfg(test)]
mod tests {
	use crate::server::room::command::room::attach_to_room;
	use crate::server::room::config::member::MemberCreateParams;
	use crate::server::room::config::room::RoomCreateParams;
	use crate::server::room::Room;
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::owner::GameObjectOwner;

	#[test]
	pub(crate) fn should_load_object_when_attach_to_room() {
		let template = RoomCreateParams::default();
		let mut room = Room::new(0, template);
		let groups_a = AccessGroups(0b100);
		let member_a = room.register_member(MemberCreateParams::stub(groups_a));
		let groups_b = AccessGroups(0b10);
		let member_b = room.register_member(MemberCreateParams::stub(groups_b));

		room.mark_as_connected_in_test(member_a).unwrap();
		room.mark_as_connected_in_test(member_b).unwrap();

		let object_a_1 = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_b), groups_a, Default::default());
		object_a_1.created = true;
		let object_a_1_id = object_a_1.id;

		// не созданный объект - не должен загрузиться
		room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_b), groups_a, Default::default());
		// другая группа + созданный объект - не должен загрузиться
		room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_b), groups_b, Default::default()).created = true;
		// другая группа - не должен загрузиться
		room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_b), groups_b, Default::default());

		attach_to_room(&mut room, member_a).unwrap();

		let mut commands = room.get_member_out_commands_for_test(member_a);
		assert!(matches!(commands.pop_front(), Some(S2CCommand::Create(c)) if c.object_id==object_a_1_id));
		assert!(matches!(commands.pop_front(), Some(S2CCommand::Created(c)) if c.object_id==object_a_1_id));
		assert!(matches!(commands.pop_front(), None));
	}

	#[test]
	pub(crate) fn should_self_object_attach_to_room() {
		let template = RoomCreateParams::default();
		let mut room = Room::new(0, template);
		let groups = AccessGroups(0b100);
		let member = room.register_member(MemberCreateParams::stub(groups));
		room.mark_as_connected_in_test(member).unwrap();

		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member), groups, Default::default());
		object.created = true;
		let object_id = object.id;

		attach_to_room(&mut room, member).unwrap();
		let mut commands = room.get_member_out_commands_for_test(member);
		assert!(matches!(commands.pop_front(), Some(S2CCommand::Create(c)) if c.object_id==object_id));

		// проверяем на второй attach, он должен сработать аналогично
		attach_to_room(&mut room, member).unwrap();
		let mut commands = room.get_member_out_commands_for_test(member);
		assert!(matches!(commands.pop_front(), Some(S2CCommand::Create(c)) if c.object_id==object_id));
	}
}
