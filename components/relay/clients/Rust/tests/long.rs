#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;

use cheetah_relay::room::template::config::Permission;
use cheetah_relay::room::types::FieldType;
use cheetah_relay_client::ffi;
use cheetah_relay_client::ffi::command::S2CMetaCommandInformationFFI;
use cheetah_relay_client::ffi::GameObjectIdFFI;
use cheetah_relay_common::constants::FieldId;

use crate::helpers::helper::*;
use crate::helpers::server::*;

pub mod helpers;

#[test]
fn should_inc() {
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());

	ffi::client::set_current_client(client1);
	let object_id = helper.create_user_object();
	ffi::command::long_value::inc_long_value(&object_id, 1, 100);
	ffi::command::long_value::inc_long_value(&object_id, 1, 100);

	ffi::client::set_current_client(client2);
	ffi::command::long_value::set_long_value_listener(listener_for_inc);
	ffi::command::room::attach_to_room();
	helper.wait_udp();
	ffi::client::receive();

	assert!(matches!(INCR.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id == 1 && *value==200 ));
}

#[test]
fn should_set() {
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());

	ffi::client::set_current_client(client1);
	let object_id = helper.create_user_object();
	ffi::command::long_value::set_long_value(&object_id, 1, 100);
	ffi::command::long_value::set_long_value(&object_id, 1, 200);

	ffi::client::set_current_client(client2);
	ffi::command::long_value::set_long_value_listener(listener_for_set);
	ffi::command::room::attach_to_room();
	helper.wait_udp();
	ffi::client::receive();

	assert!(matches!(SET.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id == 1 && *value==200 ));
}

#[test]
fn should_compare_and_set() {
	let mut builder = IntegrationTestServerBuilder::default();

	let field_id = 1;
	builder.set_permission(
		IntegrationTestServerBuilder::DEFAULT_TEMPLATE,
		field_id,
		FieldType::Long,
		IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
		Permission::Rw,
	);
	let (helper, client1, client2) = setup(builder);

	ffi::client::set_current_client(client1);
	ffi::command::long_value::set_long_value_listener(listener_for_compare_and_set);
	ffi::command::room::attach_to_room();
	let object_id = helper.create_user_object();
	helper.wait_udp();

	ffi::client::set_current_client(client2);
	ffi::command::room::attach_to_room();
	// проверяем, что установится только первое значение
	ffi::command::long_value::compare_and_set_long_value(&object_id, field_id, 0, 100, 555);
	ffi::command::long_value::compare_and_set_long_value(&object_id, field_id, 0, 200, 777);
	helper.wait_udp();
	ffi::client::set_current_client(client1);
	ffi::client::receive();
	assert!(matches!(COMPARE_AND_SET.lock().unwrap().as_ref(),Option::Some((c_field_id, value)) if *c_field_id == field_id && *value==100 ));

	// теперь второй клиент разрывает соединение
	// первый наблюдает за тем что значение поменяется на reset
	ffi::client::set_current_client(client2);
	ffi::client::destroy_client();
	helper.wait_udp();

	ffi::client::set_current_client(client1);
	ffi::client::receive();
	assert!(matches!(COMPARE_AND_SET.lock().unwrap().as_ref(),Option::Some((c_field_id, value)) if *c_field_id == field_id && *value==555 ));
}

lazy_static! {
	static ref INCR: Mutex<Option<(FieldId, i64)>> = Mutex::new(Default::default());
}

lazy_static! {
	static ref SET: Mutex<Option<(FieldId, i64)>> = Mutex::new(Default::default());
}

lazy_static! {
	static ref COMPARE_AND_SET: Mutex<Option<(FieldId, i64)>> = Mutex::new(Default::default());
}

extern "C" fn listener_for_set(_: &S2CMetaCommandInformationFFI, _object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	SET.lock().unwrap().replace((field_id, value));
}

extern "C" fn listener_for_inc(_: &S2CMetaCommandInformationFFI, _object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	INCR.lock().unwrap().replace((field_id, value));
}

extern "C" fn listener_for_compare_and_set(_: &S2CMetaCommandInformationFFI, _object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	COMPARE_AND_SET.lock().unwrap().replace((field_id, value));
}
