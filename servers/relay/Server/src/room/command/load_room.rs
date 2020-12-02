use cheetah_relay_common::commands::command::load::CreateGameObjectCommand;
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::Room;

pub fn attach_to_room(room: &mut Room, user_public_key: &UserPublicKey) {
	let mut out = Vec::new();
	match room.get_user_mut(user_public_key) {
		None => {
			log::error!("[load_room] user not found {:?}", user_public_key);
		}
		Some(user) => {
			user.attach_to_room();
			let access_group = user.template.access_groups;
			room.process_objects(&mut |o| {
				if o.access_groups.contains_any(&access_group) {
					out.push(CreateGameObjectCommand {
						object_id: o.id.clone(),
						template: o.template.clone(),
						access_groups: o.access_groups,
						fields: o.fields.clone(),
					});
				}
			});

			for command in out {
				room.send_to_user(user_public_key, S2CCommand::Create(command));
			}
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
		let mut config = RoomTemplate::default();
		let groups_a = AccessGroups(0b100);
		let user_a = config.create_user(1, groups_a);
		let groups_b = AccessGroups(0b10);
		config.create_user(2, groups_b);
		let mut room = Room::new(config);

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
