use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use lazy_static::lazy_static;

use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::logs::init_logger;
use cheetah_matches_relay_client::ffi::GameObjectIdFFI;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::helpers::helper::setup;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

///
/// Тестируем работу сервера под большой нагрузкой
///
#[test]
pub fn test() {
	init_logger();
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());
	let object_id = helper.create_user_object(client1);

	ffi::command::room::attach_to_room(client2);
	ffi::command::long_value::set_long_value_listener(client2, listener);

	let count = 50;
	for _ in 0..count {
		for _ in 0..10 {
			ffi::command::long_value::inc_long_value(client1, &object_id, 1, 1);
		}
		thread::sleep(Duration::from_millis(50));
	}

	thread::sleep(Duration::from_millis(2000));
	ffi::client::receive(client2);

	assert!(matches!(LONG_VALUE.lock().unwrap().as_ref(),
			Option::Some((id, field_id, value))
			if *id== object_id  && *field_id == 1 && *value==count*10));
}

lazy_static! {
	static ref LONG_VALUE: Mutex<Option<(GameObjectIdFFI, FieldId, i64)>> = Mutex::new(Default::default());
}

extern "C" fn listener(_: RoomMemberId, object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	LONG_VALUE.lock().unwrap().replace(((*object_id).clone(), field_id, value));
}
