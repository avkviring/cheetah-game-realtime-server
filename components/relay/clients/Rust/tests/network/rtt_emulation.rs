use std::sync::Mutex;
use std::time::Duration;

use cheetah_relay_client::ffi;
use cheetah_relay_client::ffi::command::S2CMetaCommandInformationFFI;
use cheetah_relay_client::ffi::GameObjectIdFFI;
use cheetah_relay_common::constants::FieldId;

use crate::helpers::helper::*;
use crate::helpers::server::*;

#[test]
fn test() {
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());

	ffi::client::set_current_client(client1);
	let object_id = helper.create_user_object();
	helper.wait_udp();

	ffi::client::set_current_client(client2);
	ffi::command::long_value::set_long_value_listener(listener);
	ffi::command::room::attach_to_room();
	ffi::client::set_rtt_emulation(300, 0.0);

	ffi::client::set_current_client(client1);
	ffi::command::long_value::set_long_value(&object_id, 1, 555);

	ffi::client::set_current_client(client2);
	std::thread::sleep(Duration::from_millis(100));
	ffi::client::receive();
	assert!(matches!(SET.lock().unwrap().as_ref(), Option::None));

	std::thread::sleep(Duration::from_millis(250));
	ffi::client::receive();
	assert!(matches!(SET.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id == 1 && *value==555 ));
}

lazy_static! {
	static ref SET: Mutex<Option<(FieldId, i64)>> = Mutex::new(Default::default());
}

extern "C" fn listener(_: &S2CMetaCommandInformationFFI, _object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	SET.lock().unwrap().replace((field_id, value));
}
