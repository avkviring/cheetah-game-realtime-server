use std::sync::Mutex;
use std::time::Duration;

use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::GameObjectIdFFI;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::helpers::helper::*;
use crate::helpers::server::*;

#[test]
fn test() {
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());

	let object_id = helper.create_user_object(client1);
	helper.wait_udp();

	ffi::command::long_value::set_long_value_listener(client2, listener);
	ffi::command::room::attach_to_room(client2);
	ffi::client::set_rtt_emulation(client2, 300, 0.0);

	ffi::command::long_value::set_long_value(client1, &object_id, 1, 555);

	std::thread::sleep(Duration::from_millis(200));
	ffi::client::receive(client2);
	assert!(matches!(SET.lock().unwrap().as_ref(), Option::None));

	std::thread::sleep(Duration::from_millis(250));
	ffi::client::receive(client2);
	assert!(matches!(SET.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id == 1 && *value==555 ));
}

lazy_static! {
	static ref SET: Mutex<Option<(FieldId, i64)>> = Mutex::new(Default::default());
}

extern "C" fn listener(_: RoomMemberId, _object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	SET.lock().unwrap().replace((field_id, value));
}
