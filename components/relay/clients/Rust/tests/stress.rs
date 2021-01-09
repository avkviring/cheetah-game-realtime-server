#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;

use cheetah_relay_client::ffi;
use cheetah_relay_client::ffi::command::S2CMetaCommandInformationFFI;
use cheetah_relay_client::ffi::GameObjectIdFFI;
use cheetah_relay_common::constants::FieldId;

use crate::helpers::helper::*;
use crate::helpers::server::*;

pub mod helpers;

///
/// Тестируем работу сервера под большой нагрузкой
///
#[test]
pub fn test() {
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());
	ffi::client::set_current_client(client1);
	let object_id = helper.create_user_object();

	ffi::client::set_current_client(client2);
	ffi::command::room::attach_to_room();
	ffi::command::long_value::set_long_value_listener(listener);

	ffi::client::set_current_client(client1);
	let count = 500;
	for _ in 0..count {
		ffi::command::long_value::inc_long_value(&object_id, 1, 1);
	}
	ffi::client::set_current_client(client2);
	helper.wait_udp();
	ffi::client::receive();

	assert!(matches!(LONG_VALUE.lock().unwrap().as_ref(), Option::Some((id, field_id, value)) if *id == object_id  && *field_id == 1 && *value==500));
}

lazy_static! {
	static ref LONG_VALUE: Mutex<Option<(GameObjectIdFFI, FieldId, i64)>> = Mutex::new(Default::default());
}

extern "C" fn listener(_: &S2CMetaCommandInformationFFI, object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	LONG_VALUE.lock().unwrap().replace(((*object_id).clone(), field_id, value));
}
