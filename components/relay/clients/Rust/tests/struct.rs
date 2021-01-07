#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;

use cheetah_relay::test_env::IntegrationTestServerBuider;
use cheetah_relay_client::ffi;
use cheetah_relay_client::ffi::command::S2CMetaCommandInformationFFI;
use cheetah_relay_client::ffi::{BufferFFI, GameObjectIdFFI};
use cheetah_relay_common::constants::FieldId;

use crate::helpers::*;

pub mod helpers;

///
/// Тест на создание/удаление объекта
///
#[test]
fn test() {
	let (helper, client1, client2) = setup();

	ffi::client::set_current_client(client2);
	ffi::command::structure::set_structure_listener(on_structure_listener);
	ffi::command::room::attach_to_room();
	helper.wait_first_frame();

	ffi::client::set_current_client(client1);
	let mut object_id = GameObjectIdFFI::new();
	ffi::command::object::create_object(1, IntegrationTestServerBuider::DEFAULT_ACCESS_GROUP.0, &mut object_id);
	let mut structure_buffer = BufferFFI::new();
	structure_buffer.len = 1;
	structure_buffer.buffer[0] = 100;
	let structure_field_id = 10;
	ffi::command::structure::set_structure(&object_id, structure_field_id, &structure_buffer);
	ffi::command::object::created_object(&object_id);

	helper.wait_first_frame();
	ffi::client::set_current_client(client2);
	ffi::client::receive();

	assert!(
		matches!(STRUCTURE.lock().unwrap().as_ref(),Option::Some((field_id, buffer)) if *field_id == structure_field_id && *buffer == structure_buffer )
	);
}

lazy_static! {
	static ref STRUCTURE: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
}

extern "C" fn on_structure_listener(_: &S2CMetaCommandInformationFFI, _object_id: &GameObjectIdFFI, field_id: FieldId, buffer: &BufferFFI) {
	STRUCTURE.lock().unwrap().replace((field_id, (*buffer).clone()));
}
