use std::sync::Mutex;

use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::{BufferFFI, GameObjectIdFFI};
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::helpers::helper::*;
use crate::helpers::server::*;

#[test]
fn test() {
	let builder = IntegrationTestServerBuilder::default();
	let mut helper = IntegrationTestHelper::new(builder);
	let (user1_id, user1_key) = helper.create_user();
	let (user2_id, user2_key) = helper.create_user();
	let (user3_id, user3_key) = helper.create_user();

	let client1 = helper.create_client(user1_id, user1_key);
	let client2 = helper.create_client(user2_id, user2_key);
	let client3 = helper.create_client(user3_id, user3_key);

	ffi::command::event::set_event_listener(client2, on_event_listener);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();

	ffi::command::event::set_event_listener(client3, on_event_listener);
	ffi::command::room::attach_to_room(client3);
	helper.wait_udp();

	let mut object_id = GameObjectIdFFI::default();
	ffi::command::object::create_object(
		client1,
		1,
		IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0,
		&mut object_id,
	);
	ffi::command::object::created_object(client1, &object_id);

	let mut event_buffer = BufferFFI::default();
	event_buffer.len = 1;
	event_buffer.buffer[0] = 100;
	let event_field_id = 10;

	ffi::command::event::send_target_event(client1, user2_id, &object_id, event_field_id, &event_buffer);
	helper.wait_udp();

	ffi::client::receive(client3);
	assert!(matches!(EVENT.lock().unwrap().as_ref(), None));

	ffi::client::receive(client2);
	assert!(
		matches!(EVENT.lock().unwrap().as_ref(),Option::Some((field_id, buffer)) if *field_id == event_field_id && *buffer == event_buffer )
	);
}

lazy_static! {
	static ref EVENT: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
}

extern "C" fn on_event_listener(_: RoomMemberId, _object_id: &GameObjectIdFFI, field_id: FieldId, buffer: &BufferFFI) {
	EVENT.lock().unwrap().replace((field_id, (*buffer).clone()));
}
