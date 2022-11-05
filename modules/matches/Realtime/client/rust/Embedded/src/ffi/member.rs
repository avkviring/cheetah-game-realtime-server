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
pub extern "C" fn create_member(
	server_id: ServerId,
	room_id: RoomId,
	group: u64,
	member_descriptions: &mut MemberDescription,
	on_error: extern "C" fn(*const u16),
) -> bool {
	let mut registry = REGISTRY.lock().unwrap();
	return if let Some(server) = registry.servers.get_mut(&server_id) {
		let manager = server.manager.clone();
		let member_template = MemberTemplate::new_member(AccessGroups(group), Vec::new());
		member_descriptions.private_key.copy_from_slice(&member_template.private_key.0);
		match server
			.runtime
			.block_on(async move { manager.lock().await.create_member(room_id, &member_template) })
		{
			Ok(member_id) => {
				member_descriptions.id = member_id;
				true
			}
			Err(e) => {
				on_error(widestring::U16CString::from_str(format!("{:?}", e)).unwrap().as_ptr());
				false
			}
		}
	} else {
		on_error(widestring::U16CString::from_str("Embedded server not found").unwrap().as_ptr());
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
		run_new_server(&mut result, on_server_error);
		let mut room_id = 0;
		create_room(result.id, &mut room_id, on_room_error);
		let mut member = Default::default();
		assert!(create_member(result.id, room_id, 777, &mut member, on_member_error));
		assert_eq!(member.id, 1);
	}

	pub extern "C" fn on_server_error(message: *const u16) {
		panic!("Fail create server with message {:?}", message)
	}
	pub extern "C" fn on_room_error(_: *const u16) {
		panic!("Fail create room")
	}
	pub extern "C" fn on_member_error(_: *const u16) {
		panic!("Fail create member")
	}
}
