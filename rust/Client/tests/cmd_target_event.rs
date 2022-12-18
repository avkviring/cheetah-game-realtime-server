use std::sync::Mutex;

use lazy_static::lazy_static;

use cheetah_client::ffi;
use cheetah_client::ffi::BufferFFI;
use cheetah_common::commands::field::FieldId;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::RoomMemberId;

use crate::helpers::helper::IntegrationTestHelper;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

#[test]
fn test() {
	let builder = IntegrationTestServerBuilder::default();
	let mut helper = IntegrationTestHelper::new(builder);
	let (member1_id, member1_key) = helper.create_member();
	let (member2_id, member2_key) = helper.create_member();
	let (member3_id, member3_key) = helper.create_member();

	let client1 = helper.create_client(member1_id, &member1_key);
	let client2 = helper.create_client(member2_id, &member2_key);
	let client3 = helper.create_client(member3_id, &member3_key);

	ffi::command::event::set_event_listener(client2, on_event_listener);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();

	ffi::command::event::set_event_listener(client3, on_event_listener);
	ffi::command::room::attach_to_room(client3);
	helper.wait_udp();

	let mut object_id = GameObjectId::default();
	ffi::command::object::create_object(client1, 1, IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0, &mut object_id);
	ffi::command::object::created_object(client1, &object_id, false, &BufferFFI::default());

	let mut event_buffer = BufferFFI {
		len: 1,
		..Default::default()
	};
	event_buffer.buffer[0] = 100;
	let event_field_id = 10;

	ffi::command::event::send_target_event(client1, member2_id, &object_id, event_field_id, &event_buffer);
	helper.wait_udp();

	ffi::client::receive(client3);
	assert!(matches!(EVENT.lock().unwrap().as_ref(), None));

	ffi::client::receive(client2);
	assert!(matches!(EVENT.lock().unwrap().as_ref(),Some((field_id, buffer)) if *field_id == event_field_id && *buffer == event_buffer ));
}

lazy_static! {
	static ref EVENT: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
}

extern "C" fn on_event_listener(_: RoomMemberId, _object_id: &GameObjectId, field_id: FieldId, buffer: &BufferFFI) {
	EVENT.lock().unwrap().replace((field_id, (*buffer).clone()));
}
