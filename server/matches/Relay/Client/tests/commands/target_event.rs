use std::sync::Mutex;

use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::command::S2CMetaCommandInformationFFI;
use cheetah_matches_relay_client::ffi::{BufferFFI, GameObjectIdFFI};
use cheetah_matches_relay_common::constants::FieldId;

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

	ffi::client::set_current_client(client2);
	ffi::command::event::set_event_listener(on_event_listener);
	ffi::command::room::attach_to_room();
	helper.wait_udp();

	ffi::client::set_current_client(client3);
	ffi::command::event::set_event_listener(on_event_listener);
	ffi::command::room::attach_to_room();
	helper.wait_udp();

	ffi::client::set_current_client(client1);
	let mut object_id = GameObjectIdFFI::new();
	ffi::command::object::create_object(1, IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0, &mut object_id);
	ffi::command::object::created_object(&object_id);

	let mut event_buffer = BufferFFI::new();
	event_buffer.len = 1;
	event_buffer.buffer[0] = 100;
	let event_field_id = 10;

	ffi::command::event::send_target_event(user2_id, &object_id, event_field_id, &event_buffer);
	helper.wait_udp();

	ffi::client::set_current_client(client3);
	ffi::client::receive();
	assert!(matches!(EVENT.lock().unwrap().as_ref(), None));

	ffi::client::set_current_client(client2);
	ffi::client::receive();
	assert!(
		matches!(EVENT.lock().unwrap().as_ref(),Option::Some((field_id, buffer)) if *field_id == event_field_id && *buffer == event_buffer )
	);
}

lazy_static! {
	static ref EVENT: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
}

extern "C" fn on_event_listener(
	_: &S2CMetaCommandInformationFFI,
	_object_id: &GameObjectIdFFI,
	field_id: FieldId,
	buffer: &BufferFFI,
) {
	EVENT.lock().unwrap().replace((field_id, (*buffer).clone()));
}
