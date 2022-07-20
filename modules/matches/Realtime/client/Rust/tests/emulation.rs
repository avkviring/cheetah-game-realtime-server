use std::sync::Mutex;
use std::time::Duration;

use lazy_static::lazy_static;

use cheetah_matches_realtime_client::ffi;
use cheetah_matches_realtime_client::ffi::channel::Channel;
use cheetah_matches_realtime_client::ffi::GameObjectIdFFI;
use cheetah_matches_realtime_common::constants::FieldId;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::helpers::helper::setup;

pub mod helpers;

fn should_drop() {
	let (helper, [client1, client2]) = setup(Default::default());

	let object_id = helper.create_member_object(client1);
	helper.wait_udp();

	ffi::command::long_value::set_long_value_listener(client2, should_drop_listener);
	ffi::command::room::attach_to_room(client2);
	ffi::client::set_drop_emulation(client2, 0.1, 0);
	helper.wait_udp();

	ffi::channel::set_channel(client1, Channel::ReliableOrdered, 0);
	const COMMAND_COUNTS: usize = 200000;
	for _ in 0..COMMAND_COUNTS {
		ffi::command::long_value::inc_long_value(client1, &object_id, 1, 1);
	}
	helper.wait_udp();
	ffi::client::receive(client2);

	match SHOULD_DROP_SET.lock().unwrap().as_ref() {
		None => assert!(false, "SHOULD_DROP_SET is None"),
		Some(counter) => {
			assert!(
				counter.1 < COMMAND_COUNTS as i64,
				"counter should less 200_000 {}",
				counter.1
			);
			assert!(counter.1 > 0, "counter should more zero {}", counter.1);
		}
	}
}

lazy_static! {
	static ref SHOULD_DROP_SET: Mutex<Option<(FieldId, i64)>> = Mutex::new(Default::default());
}

extern "C" fn should_drop_listener(
	_: RoomMemberId,
	_object_id: &GameObjectIdFFI,
	field_id: FieldId,
	value: i64,
) {
	SHOULD_DROP_SET.lock().unwrap().replace((field_id, value));
}

#[test]
fn should_rtt_emulation() {
	let (helper, [client1, client2]) = setup(Default::default());

	let object_id = helper.create_member_object(client1);
	helper.wait_udp();

	ffi::command::long_value::set_long_value_listener(client2, should_rtt_listener);
	ffi::command::room::attach_to_room(client2);
	ffi::client::set_rtt_emulation(client2, 300, 0.0);

	ffi::command::long_value::set_long_value(client1, &object_id, 1, 555);

	std::thread::sleep(Duration::from_millis(200));
	ffi::client::receive(client2);
	assert!(matches!(
		SHOULD_RTT_SET.lock().unwrap().as_ref(),
		Option::None
	));

	std::thread::sleep(Duration::from_millis(250));
	ffi::client::receive(client2);
	assert!(
		matches!(SHOULD_RTT_SET.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id == 1 && *value==555 )
	);
}

lazy_static! {
	static ref SHOULD_RTT_SET: Mutex<Option<(FieldId, i64)>> = Mutex::new(Default::default());
}

extern "C" fn should_rtt_listener(
	_: RoomMemberId,
	_object_id: &GameObjectIdFFI,
	field_id: FieldId,
	value: i64,
) {
	SHOULD_RTT_SET.lock().unwrap().replace((field_id, value));
}
