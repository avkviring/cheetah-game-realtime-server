use cheetah_common::room::RoomMemberId;
use std::rc::Rc;

use crate::room::command::ServerCommandError;
use crate::room::object::CreateCommandsCollector;
use crate::room::Room;

pub fn attach_to_room(room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	let member = room.get_member_mut(&member_id)?;
	member.attached = true;
	let access_group = member.template.groups;
	let command_collector_rc = Rc::clone(&room.tmp_command_collector);
	let mut command_collector = (*command_collector_rc).borrow_mut();
	command_collector.clear();
	room.objects
		.iter()
		.filter(|(_, o)| o.created)
		.filter(|(_, o)| o.access_groups.contains_any(&access_group))
		.map(|(_, o)| {
			let mut commands = CreateCommandsCollector::new();
			o.collect_create_commands(&mut commands, member_id);
			(o.template_id, commands)
		})
		.clone()
		.for_each(|v| command_collector.push(v));

	for (template, commands) in command_collector.iter() {
		room.send_to_member(&member_id, *template, commands.as_slice())?;
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
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::room::command::room::attach_to_room;
	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::room::Room;

	#[test]
	pub(crate) fn should_load_object_when_attach_to_room() {
		let template = RoomTemplate::default();
		let mut room = Room::from_template(template);
		let groups_a = AccessGroups(0b100);
		let member_a = room.register_member(MemberTemplate::stub(groups_a));
		let groups_b = AccessGroups(0b10);
		let member_b = room.register_member(MemberTemplate::stub(groups_b));

		room.test_mark_as_connected(member_a).unwrap();
		room.test_mark_as_connected(member_b).unwrap();

		let object_a_1 = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_b), groups_a);
		object_a_1.created = true;
		let object_a_1_id = object_a_1.id;

		// не созданный объект - не должен загрузиться
		room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_b), groups_a);
		// другая группа + созданный объект - не должен загрузиться
		room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_b), groups_b).created = true;
		// другая группа - не должен загрузиться
		room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_b), groups_b);

		attach_to_room(&mut room, member_a).unwrap();

		let mut commands = room.test_get_member_out_commands(member_a);
		assert!(matches!(commands.pop_front(), Some(S2CCommand::Create(c)) if c.object_id==object_a_1_id));
		assert!(matches!(commands.pop_front(), Some(S2CCommand::Created(c)) if c.object_id==object_a_1_id));
		assert!(matches!(commands.pop_front(), None));
	}
}
