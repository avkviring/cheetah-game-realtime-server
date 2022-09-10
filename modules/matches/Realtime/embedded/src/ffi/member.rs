use cheetah_matches_realtime::room::template::config::MemberTemplate;
use cheetah_matches_realtime_common::room::access::AccessGroups;
use cheetah_matches_realtime_common::room::{RoomId, RoomMemberId};

use crate::ffi::{ServerId, REGISTRY};

#[derive(Debug, Default)]
#[repr(C)]
pub struct MemberDescription {
	id: RoomMemberId,
	private_key: [u8; 32],
}

#[no_mangle]
pub extern "C" fn create_member(server_id: ServerId, room_id: RoomId, group: u64, member_descriptions: &mut MemberDescription) -> bool {
	let mut registry = REGISTRY.lock().unwrap();
	return if let Some(server) = registry.servers.get_mut(&server_id) {
		let manager = server.manager.clone();
		let member_template = MemberTemplate::new_member(AccessGroups(group), Vec::new());
		member_descriptions.private_key.copy_from_slice(&member_template.private_key.0);
		match server
			.runtime
			.block_on(async move { manager.lock().await.create_member(room_id, member_template) })
		{
			Ok(member_id) => {
				member_descriptions.id = member_id;
				true
			}
			Err(_) => false,
		}
	} else {
		false
	};
}

#[cfg(test)]
mod test {
	use crate::ffi::member::create_member;
	use crate::ffi::room::create_room;
	use crate::ffi::server::{run_new_server, EmbeddedServerDescription};

	#[test]
	pub fn should_create_member() {
		let mut result = EmbeddedServerDescription::default();
		run_new_server(&mut result);
		let mut room_id = 0;
		create_room(result.id, &mut room_id);
		let mut member = Default::default();
		assert!(create_member(result.id, room_id, 777, &mut member));
		assert_eq!(member.id, 1);
	}
}
