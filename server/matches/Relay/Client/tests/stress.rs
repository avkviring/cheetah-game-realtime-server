use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use lazy_static::lazy_static;

use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::channel::Channel;
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
pub fn stress_test() {
	init_logger();
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());
	let object_id = helper.create_user_object(client1);
	thread::sleep(Duration::from_millis(1000));

	ffi::command::room::attach_to_room(client2);
	ffi::command::long_value::set_long_value_listener(client2, listener);
	ffi::client::set_drop_emulation(client1, 0.1, 1);
	ffi::client::set_rtt_emulation(client1, 10, 0.0);
	ffi::client::set_drop_emulation(client2, 0.1, 1);
	ffi::client::set_rtt_emulation(client2, 10, 0.0);
	let count = 5099;
	let mut send_inc_long_count = 0;
	for i in 0..count {
		ffi::channel::set_channel(client1, Channel::UnreliableUnordered, 0);
		ffi::command::float_value::inc_double_value(client1, &object_id, 2, 1.0);
		if i % 100 == 0 {
			ffi::channel::set_channel(client1, Channel::ReliableSequence, 1);
			ffi::command::long_value::inc_long_value(client1, &object_id, 1, 1);
			send_inc_long_count += 1;
		}
		ffi::client::receive(client1);
		ffi::client::receive(client2);
		thread::sleep(Duration::from_millis(5));
	}

	thread::sleep(Duration::from_millis(2000));
	ffi::client::receive(client2);

	tracing::info!("result {:?} {:?}", send_inc_long_count, LONG_VALUE.lock().unwrap().as_ref());

	assert!(matches!(LONG_VALUE.lock().unwrap().as_ref(),
			Option::Some((id, field_id, value))
			if *id== object_id  && *field_id == 1 && *value==send_inc_long_count));
}

lazy_static! {
	static ref LONG_VALUE: Mutex<Option<(GameObjectIdFFI, FieldId, i64)>> = Mutex::new(Default::default());
}

extern "C" fn listener(_: RoomMemberId, object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	LONG_VALUE.lock().unwrap().replace(((*object_id).clone(), field_id, value));
}
