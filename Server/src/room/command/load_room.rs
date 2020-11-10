use cheetah_relay_common::commands::hash::UserPublicKey;

use crate::room::Room;

pub fn load_room(room: &mut dyn Room, user_public_key: &UserPublicKey) {
	//room.process_objects()
	
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
		
		assert!(matches!(room.out_command.pop_back(), Some((.., S2CCommandUnion::Create(c))) if c.object_id==object_a_1));
		assert!(matches!(room.out_command.pop_back(), Some((.., S2CCommandUnion::Create(c))) if c.object_id==object_a_2));
		assert!(matches!(room.out_command.pop_back(), None));
	}
}


