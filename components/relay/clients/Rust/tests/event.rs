#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;

use cheetah_relay_client::ffi;
use cheetah_relay_client::ffi::command::S2CMetaCommandInformationFFI;
use cheetah_relay_client::ffi::{BufferFFI, GameObjectIdFFI};
use cheetah_relay_common::constants::FieldId;

use crate::helpers::helper::*;
use crate::helpers::server::*;

pub mod helpers;

#[test]
fn test() {
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());

	ffi::client::set_current_client(client2);
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
	ffi::command::event::send_event(&object_id, event_field_id, &event_buffer);

	helper.wait_udp();
	ffi::client::set_current_client(client2);
	ffi::client::receive();

	assert!(matches!(EVENT.lock().unwrap().as_ref(),Option::Some((field_id, buffer)) if *field_id == event_field_id && *buffer == event_buffer ));
}

lazy_static! {
	static ref EVENT: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
}

extern "C" fn on_event_listener(_: &S2CMetaCommandInformationFFI, _object_id: &GameObjectIdFFI, field_id: FieldId, buffer: &BufferFFI) {
	EVENT.lock().unwrap().replace((field_id, (*buffer).clone()));
}
