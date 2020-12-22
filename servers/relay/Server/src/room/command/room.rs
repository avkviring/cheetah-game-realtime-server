use cheetah_relay_common::constants::GameObjectTemplateId;
use cheetah_relay_common::room::UserId;

use crate::room::object::S2CommandWithFieldInfo;
use crate::room::Room;

pub fn attach_to_room(room: &mut Room, user_id: &UserId) {
	match room.get_user_mut(user_id) {
		None => {
			log::error!("[load_room] user not found {:?}", user_id);
		}
		Some(user) => {
			user.attach_to_room();
			let access_group = user.template.access_groups;
			let commands_by_object: Vec<(GameObjectTemplateId, Vec<S2CommandWithFieldInfo>)> = room
				.objects
				.iter()
				.filter(|(_, o)| o.access_groups.contains_any(&access_group))
				.map(|(_, o)| {
					let mut commands = Vec::new();
					o.collect_create_commands(&mut commands);
					(o.template, commands)
				})
				.collect();

			for (template, commands) in commands_by_object {
				room.send_to_user(&user_id, template, commands);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::access::AccessGroups;

	use crate::room::command::room::attach_to_room;
	use crate::room::template::config::RoomTemplate;
	use crate::room::Room;

	#[test]
	pub fn test() {
		let mut template = RoomTemplate::default();
		let groups_a = AccessGroups(0b100);
		let user_a = template.configure_user(1, groups_a);
		let groups_b = AccessGroups(0b10);
		let user_b = template.configure_user(3, groups_b);
		let mut room = Room::from_template(template);

		room.mark_as_connected(&user_a);
		room.mark_as_connected(&user_b);

		let object_a_1 = room.create_object(&user_b, groups_a).id.clone();
		let object_a_2 = room.create_object(&user_b, groups_a).id.clone();
		room.create_object(&user_b, groups_b);
		room.create_object(&user_b, groups_b);

		attach_to_room(&mut room, &user_a);

		let mut commands = room.get_user_out_commands(&user_a);
		assert!(matches!(commands.pop_front(), Some(S2CCommand::Create(c)) if c.object_id==object_a_1));
		assert!(matches!(commands.pop_front(), Some(S2CCommand::Create(c)) if c.object_id==object_a_2));
		assert!(matches!(commands.pop_front(), None));
	}
}
