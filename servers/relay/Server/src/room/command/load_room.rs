use cheetah_relay_common::room::UserPublicKey;

use crate::room::Room;

pub fn attach_to_room(room: &mut Room, user_public_key: &UserPublicKey) {
	match room.get_user_mut(user_public_key) {
		None => {
			log::error!("[load_room] user not found {:?}", user_public_key);
		}
		Some(user) => {
			user.attach_to_room();
			let access_group = user.template.access_groups;
			let mut commands = Vec::new();
			room.process_objects(&mut |o| {
				if o.access_groups.contains_any(&access_group) {
					o.collect_create_commands(&mut commands);
				}
			});
			room.send_to_user(user_public_key, commands);
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::access::AccessGroups;

	use crate::room::command::load_room::attach_to_room;
	use crate::room::template::RoomTemplate;
	use crate::room::Room;

	#[test]
	pub fn test() {
		let mut template = RoomTemplate::default();
		let groups_a = AccessGroups(0b100);
		let user_a = template.create_user(1, groups_a);
		let groups_b = AccessGroups(0b10);
		template.create_user(2, groups_b);
		let mut room = Room::new(template, Default::default());

		let object_a_1 = room.create_object_with_access_groups(groups_a).id.clone();
		let object_a_2 = room.create_object_with_access_groups(groups_a).id.clone();
		room.create_object_with_access_groups(groups_b);
		room.create_object_with_access_groups(groups_b);

		attach_to_room(&mut room, &user_a);

		let commands = &mut room.out_commands_by_users.get_mut(&user_a).unwrap();
		assert!(matches!(commands.pop_back(), Some(S2CCommand::Create(c)) if c.object_id==object_a_1));
		assert!(matches!(commands.pop_back(), Some(S2CCommand::Create(c)) if c.object_id==object_a_2));
		assert!(matches!(commands.pop_back(), None));
	}
}
