use std::sync::Mutex;

use lazy_static::lazy_static;

use cheetah_matches_realtime_client::ffi;
use cheetah_matches_realtime_client::ffi::GameObjectIdFFI;
use cheetah_matches_realtime_common::constants::FieldId;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::helpers::helper::setup;

pub mod helpers;

#[test]
fn should_inc() {
	let (helper, [client1, client2]) = setup(Default::default());

	let object_id = helper.create_member_object(client1);
	ffi::command::float_value::inc_double_value(client1, &object_id, 1, 100.0);
	ffi::command::float_value::inc_double_value(client1, &object_id, 1, 100.0);

	ffi::command::float_value::set_double_value_listener(client2, listener_for_inc);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();
	ffi::client::receive(client2);

	assert!(
		matches!(INCR.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id 
		== 1 && (*value - 200.0).abs() < 0.001 )
	);
}

#[test]
fn should_set() {
	let (helper, [client1, client2]) = setup(Default::default());

	let object_id = helper.create_member_object(client1);
	ffi::command::float_value::set_double_value(client1, &object_id, 1, 100.0);
	ffi::command::float_value::set_double_value(client1, &object_id, 1, 200.0);

	ffi::command::float_value::set_double_value_listener(client2, listener_for_set);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();
	ffi::client::receive(client2);

	assert!(
		matches!(SET.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id ==
		1 && (*value - 200.0).abs() < 0.001 )
	);
}

lazy_static! {
	static ref INCR: Mutex<Option<(FieldId, f64)>> = Mutex::new(Default::default());
}

lazy_static! {
	static ref SET: Mutex<Option<(FieldId, f64)>> = Mutex::new(Default::default());
}

extern "C" fn listener_for_set(
	_: RoomMemberId,
	_object_id: &GameObjectIdFFI,
	field_id: FieldId,
	value: f64,
) {
	SET.lock().unwrap().replace((field_id, value));
}

extern "C" fn listener_for_inc(
	_: RoomMemberId,
	_object_id: &GameObjectIdFFI,
	field_id: FieldId,
	value: f64,
) {
	INCR.lock().unwrap().replace((field_id, value));
}
