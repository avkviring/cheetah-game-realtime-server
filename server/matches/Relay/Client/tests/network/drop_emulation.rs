use std::sync::Mutex;

use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::channel::Channel;
use cheetah_matches_relay_client::ffi::command::S2CMetaCommandInformationFFI;
use cheetah_matches_relay_client::ffi::GameObjectIdFFI;
use cheetah_matches_relay_common::constants::FieldId;

use crate::helpers::helper::*;
use crate::helpers::server::*;

#[test]
fn test() {
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());

	let object_id = helper.create_user_object(client1);
	helper.wait_udp();

	ffi::command::long_value::set_long_value_listener(client2, listener);
	ffi::command::room::attach_to_room(client2);
	ffi::client::set_drop_emulation(client2, 0.1, 0);

	ffi::channel::set_channel(client1, Channel::UnreliableUnordered, 0);
	for _ in 0..20000 {
		ffi::command::long_value::inc_long_value(client1, &object_id, 1, 1);
	}
	helper.wait_udp();
	helper.wait_udp();
	helper.wait_udp();
	ffi::client::receive(client2);
	println!("value {:?}", SET.lock().unwrap().as_ref());
	assert!(
		matches!(SET.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id ==
		1 && *value<20000)
	);
	assert!(matches!(SET.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id == 1 && *value>0 ));
}

lazy_static! {
	static ref SET: Mutex<Option<(FieldId, i64)>> = Mutex::new(Default::default());
}

extern "C" fn listener(_: &S2CMetaCommandInformationFFI, _object_id: &GameObjectIdFFI, field_id: FieldId, value: i64) {
	SET.lock().unwrap().replace((field_id, value));
}
