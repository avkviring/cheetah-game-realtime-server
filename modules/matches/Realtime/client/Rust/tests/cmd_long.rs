use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

use cheetah_matches_realtime::room::template::config::Permission;
use cheetah_matches_realtime_client::ffi;
use cheetah_matches_realtime_client::ffi::GameObjectIdFFI;
use cheetah_matches_realtime_common::commands::FieldType;
use cheetah_matches_realtime_common::constants::FieldId;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::helpers::helper::setup;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

#[test]
fn should_inc() {
	let (helper, [client1, client2]) = setup(Default::default());

	let object_id = helper.create_member_object(client1);
	ffi::command::long_value::inc_long_value(client1, &object_id, 1, 100);
	ffi::command::long_value::inc_long_value(client1, &object_id, 1, 100);

	ffi::command::long_value::set_long_value_listener(client2, listener_for_inc);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();
	ffi::client::receive(client2);

	assert!(matches!(INCR.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id == 1 && *value==200 ));
}

#[test]
fn should_set() {
	let (helper, [client1, client2]) = setup(Default::default());

	let object_id = helper.create_member_object(client1);
	ffi::command::long_value::set_long_value(client1, &object_id, 1, 100);
	ffi::command::long_value::set_long_value(client1, &object_id, 1, 200);

	ffi::command::long_value::set_long_value_listener(client2, listener_for_set);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();
	ffi::client::receive(client2);

	assert!(matches!(SET.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id == 1 && *value==200 ));
}

#[test]
fn should_compare_and_set() {
	let mut builder = IntegrationTestServerBuilder::default();

	let field_id_with_reset = 1;
	let field_id = 2;

	// устанавливаем RW для получения команды с сервера на клиента источника команды
	builder.set_permission(
		IntegrationTestServerBuilder::DEFAULT_TEMPLATE,
		field_id_with_reset,
		FieldType::Long,
		IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
		Permission::Rw,
	);

	builder.set_permission(
		IntegrationTestServerBuilder::DEFAULT_TEMPLATE,
		field_id,
		FieldType::Long,
		IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
		Permission::Rw,
	);
	let (helper, [client1, client2]) = setup(builder);

	ffi::command::long_value::set_long_value_listener(client1, listener_for_compare_and_set);
	ffi::command::room::attach_to_room(client1);
	let object_id = helper.create_member_object(client1);
	helper.wait_udp();

	ffi::command::room::attach_to_room(client2);
	// проверяем, что установится только первое значение
	ffi::command::long_value::compare_and_set_long_value(client2, &object_id, field_id_with_reset, 0, 100, true, 555);
	ffi::command::long_value::compare_and_set_long_value(client2, &object_id, field_id, 0, 200, false, 0);
	ffi::command::long_value::compare_and_set_long_value(client2, &object_id, field_id_with_reset, 0, 200, true, 777);
	helper.wait_udp();

	ffi::client::receive(client1);
	assert_eq!(*COMPARE_AND_SET.lock().unwrap().get(&field_id_with_reset).unwrap(), 100);
	assert_eq!(*COMPARE_AND_SET.lock().unwrap().get(&field_id).unwrap(), 200);

	// теперь второй клиент разрывает соединение
	// первый наблюдает за тем что значение поменяется на reset
	ffi::client::destroy_client(client2);
	helper.wait_udp();

	ffi::client::receive(client1);
	assert_eq!(*COMPARE_AND_SET.lock().unwrap().get(&field_id_with_reset).unwrap(), 555);
}

lazy_static! {
	static ref INCR: Mutex<Option<(FieldId, i64)>> = Mutex::new(Default::default());
}

lazy_static! {
	static ref SET: Mutex<Option<(FieldId, i64)>> = Mutex::new(Default::default());
}

lazy_static! {
	static ref COMPARE_AND_SET: Mutex<HashMap<FieldId, i64>> = Mutex::new(Default::default());
}

extern "C" fn listener_for_set(_: RoomMemberId, _object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	SET.lock().unwrap().replace((field_id, value));
}

extern "C" fn listener_for_inc(_: RoomMemberId, _object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	INCR.lock().unwrap().replace((field_id, value));
}

extern "C" fn listener_for_compare_and_set(_: RoomMemberId, _object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	COMPARE_AND_SET.lock().unwrap().insert(field_id, value);
}
