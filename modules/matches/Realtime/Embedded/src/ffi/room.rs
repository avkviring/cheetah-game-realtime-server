use cheetah_matches_realtime::room::template::config::RoomTemplate;
use cheetah_matches_realtime_common::room::RoomId;

use crate::ffi::{ServerId, REGISTRY};

#[no_mangle]
pub extern "C" fn create_room(server_id: ServerId, room_id: &mut RoomId) -> bool {
	let mut registry = REGISTRY.lock().unwrap();
	return if let Some(server) = registry.servers.get_mut(&server_id) {
		let manager = server.manager.clone();
		match server
			.runtime
			.block_on(async move { manager.lock().await.create_room(RoomTemplate::default()) })
		{
			Ok(create_room_id) => {
				*room_id = create_room_id;
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
	use crate::ffi::room::create_room;
	use crate::ffi::server::{run_new_server, EmbeddedServerDescription};

	#[test]
	pub fn should_create_room() {
		let mut result = EmbeddedServerDescription::default();
		run_new_server(&mut result);
		let mut room_id = 0;
		assert!(create_room(result.id, &mut room_id));
		assert_eq!(room_id, 1)
	}
}
