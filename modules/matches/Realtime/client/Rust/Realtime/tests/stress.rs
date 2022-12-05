use std::sync::Mutex;
use std::time::Duration;
use std::{panic, thread};

use lazy_static::lazy_static;

use cheetah_matches_realtime_client::ffi;
use cheetah_matches_realtime_client::ffi::channel::Channel;
use cheetah_matches_realtime_client::ffi::logs::{init_logger, set_max_log_level, LogLevel};
use cheetah_matches_realtime_client::ffi::GameObjectIdFFI;
use cheetah_matches_realtime_common::commands::field::FieldId;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::helpers::helper::setup;

pub mod helpers;

#[test]
pub fn stress_create_lot_of_objects_test() {
	const COUNT_OBJECTS: usize = 100;
	init_logger();
	set_max_log_level(LogLevel::Warn);
	let (helper, [client1, client2]) = setup(Default::default());
	ffi::command::object::set_create_object_listener(client2, on_object_create);
	ffi::command::room::attach_to_room(client1);
	ffi::command::room::attach_to_room(client2);
	thread::sleep(Duration::from_secs(1));
	for _ in 0..COUNT_OBJECTS {
		helper.create_member_object(client1);
	}
	thread::sleep(Duration::from_secs(1));
	ffi::client::receive(client1);
	ffi::client::receive(client2);
	assert_eq!(*CREATE_OBJECT_ID.lock().unwrap(), COUNT_OBJECTS);
}

lazy_static! {
	static ref CREATE_OBJECT_ID: Mutex<usize> = Mutex::new(Default::default());
}
extern "C" fn on_object_create(_object_id: &GameObjectIdFFI, _: u16) {
	*CREATE_OBJECT_ID.lock().unwrap() += 1;
}

///
/// Тестируем работу сервера под большой нагрузкой
///
//#[test]
#[allow(clippy::print_stderr)]
pub fn stress_test() {
	init_logger();
	let (helper, [client1, client2]) = setup(Default::default());

	panic::set_hook(Box::new(move |panic_info| {
		eprintln!("{panic_info}");
	}));

	let object_id = helper.create_member_object(client1);
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

	let result = LONG_VALUE.lock();
	let result = result.unwrap();
	let result: Option<&(GameObjectIdFFI, FieldId, i64)> = result.as_ref();
	assert!(result.is_some());
	let result: &(GameObjectIdFFI, FieldId, i64) = result.unwrap();
	assert_eq!(result.0, object_id);
	assert_eq!(result.1, 1);
	assert_eq!(result.2, send_inc_long_count);
}

lazy_static! {
	static ref LONG_VALUE: Mutex<Option<(GameObjectIdFFI, FieldId, i64)>> = Mutex::new(Default::default());
}

extern "C" fn listener(_: RoomMemberId, object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	LONG_VALUE.lock().unwrap().replace(((*object_id).clone(), field_id, value));
}
