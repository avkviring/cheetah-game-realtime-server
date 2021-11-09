use std::sync::Mutex;

use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::command::S2CMetaCommandInformationFFI;
use cheetah_matches_relay_client::ffi::{BufferFFI, GameObjectIdFFI};
use cheetah_matches_relay_common::constants::FieldId;

use crate::helpers::helper::*;
use crate::helpers::server::*;

#[test]
fn test() {
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());

	ffi::client::set_current_client(client2);
	ffi::command::structure::set_structure_listener(on_structure_listener);
	ffi::command::room::attach_to_room();
	helper.wait_udp();

	ffi::client::set_current_client(client1);
	let object_id = helper.create_user_object();
	let structure_buffer = BufferFFI::from(vec![100]);
	let structure_field_id = 10;
	ffi::command::structure::set_structure(&object_id, structure_field_id, &structure_buffer);

	helper.wait_udp();
	ffi::client::set_current_client(client2);
	ffi::client::receive();

	assert!(
		matches!(STRUCTURE.lock().unwrap().as_ref(),Option::Some((field_id, buffer)) if *field_id == structure_field_id && *buffer == structure_buffer )
	);
}

lazy_static! {
	static ref STRUCTURE: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
}

extern "C" fn on_structure_listener(
	_: &S2CMetaCommandInformationFFI,
	_object_id: &GameObjectIdFFI,
	field_id: FieldId,
	buffer: &BufferFFI,
) {
	STRUCTURE.lock().unwrap().replace((field_id, (*buffer).clone()));
}
