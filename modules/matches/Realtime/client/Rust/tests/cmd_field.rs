use std::sync::Mutex;

use lazy_static::lazy_static;

use cheetah_matches_realtime_client::ffi;
use cheetah_matches_realtime_client::ffi::{FieldTypeFFI, GameObjectIdFFI};
use cheetah_matches_realtime_common::constants::FieldId;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::helpers::helper::setup;

pub mod helpers;

#[test]
fn should_delete_field_ffi() {
	let (helper, [client1, client2]) = setup(Default::default());

	let object_id = helper.create_member_object(client1);
	ffi::command::field::set_delete_field_listener(client2, delete_listener);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();
	ffi::command::field::delete_field(client1, &object_id, 1, FieldTypeFFI::Long);
	helper.wait_udp();
	ffi::client::receive(client2);

	assert!(
		matches!(DELETED_FIELD.lock().unwrap().as_ref(),Option::Some((field_id, field_type)) if 
			*field_id ==1 && *field_type==FieldTypeFFI::Long )
	);
}

#[test]
fn should_allow_fields_with_different_types_but_same_id() {
	let (helper, [client1, client2]) = setup(Default::default());

	let object_id = helper.create_member_object(client1);
	ffi::command::float_value::set_double_value(client1, &object_id, 1, 100.0);
	ffi::command::long_value::set_long_value(client1, &object_id, 1, 50);

	ffi::command::float_value::set_double_value_listener(client2, set_listener);
	ffi::command::long_value::set_long_value_listener(client2, set_listener);

	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();
	ffi::client::receive(client2);

	assert_eq!(SET_FIELDS.lock().unwrap().as_ref(), [1, 1]);
}

lazy_static! {
	static ref DELETED_FIELD: Mutex<Option<(FieldId, FieldTypeFFI)>> =
		Mutex::new(Default::default());
}

lazy_static! {
	static ref SET_FIELDS: Mutex<Vec<FieldId>> = Mutex::new(Default::default());
}

extern "C" fn delete_listener(
	_: RoomMemberId,
	_object_id: &GameObjectIdFFI,
	field_id: FieldId,
	field_type: FieldTypeFFI,
) {
	DELETED_FIELD
		.lock()
		.unwrap()
		.replace((field_id, field_type));
}

extern "C" fn set_listener<T>(
	_: RoomMemberId,
	_object_id: &GameObjectIdFFI,
	field_id: FieldId,
	_: T,
) {
	SET_FIELDS.lock().unwrap().push(field_id);
}
