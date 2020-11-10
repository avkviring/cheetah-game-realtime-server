use cheetah_relay_common::commands::command::load::CreateGameObjectCommand;
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::{Room, User};

pub fn load_room(room: &mut dyn Room, user_public_key: &UserPublicKey) {
	let mut out = Vec::new();
	match room.get_user(user_public_key) {
		None => {
			log::error!("load_room user not found {:?}", user_public_key);
		}
		Some(user) => {
			let access_group = user.access_groups;
			
			
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
				room.send_to_user(user_public_key, S2CCommandUnion::Create(command));
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::S2CCommandUnion;
	use cheetah_relay_common::room::access::AccessGroups;
	
	use crate::room::command::load_room::load_room;
	use crate::room::tests::RoomStub;
	
	#[test]
	pub fn test() {
		let mut room = RoomStub::new();
		let groups_a = AccessGroups(0b100);
		let user_a = room.create_user(groups_a);
		let object_a_1 = room.create_object_with_access_groups(groups_a).id.clone();
		let object_a_2 = room.create_object_with_access_groups(groups_a).id.clone();
		
		let groups_b = AccessGroups(0b10);
		let user_b = room.create_user(groups_b);
		let object_b_1 = room.create_object_with_access_groups(groups_b).id.clone();
		let object_b_2 = room.create_object_with_access_groups(groups_b).id.clone();
		
		load_room(&mut room, &user_a);
		
		let commands = &mut room.out_commands_by_users.get_mut(&user_a).unwrap();
		assert!(matches!(commands.pop_back(), Some(S2CCommandUnion::Create(c)) if c.object_id==object_a_1));
		assert!(matches!(commands.pop_back(), Some(S2CCommandUnion::Create(c)) if c.object_id==object_a_2));
		assert!(matches!(commands.pop_back(), None));
	}
}


