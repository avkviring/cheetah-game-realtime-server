use std::sync::Mutex;

use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::{BufferFFI, GameObjectIdFFI};
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::helpers::helper::setup;

use lazy_static::lazy_static;

pub mod helpers;

#[test]
fn test() {
	let (helper, [client1, client2]) = setup(Default::default());

	ffi::command::structure::set_structure_listener(client2, on_structure_listener);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();

	let object_id = helper.create_member_object(client1);
	let structure_buffer = BufferFFI::from(vec![100]);
	let structure_field_id = 10;
	ffi::command::structure::set_structure(client1, &object_id, structure_field_id, &structure_buffer);

	helper.wait_udp();
	ffi::client::receive(client2);

	assert!(
		matches!(STRUCTURE.lock().unwrap().as_ref(),Option::Some((field_id, buffer)) if *field_id == structure_field_id && *buffer == structure_buffer )
	);
}

lazy_static! {
	static ref STRUCTURE: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
}

extern "C" fn on_structure_listener(_: RoomMemberId, _object_id: &GameObjectIdFFI, field_id: FieldId, buffer: &BufferFFI) {
	STRUCTURE.lock().unwrap().replace((field_id, (*buffer).clone()));
}
