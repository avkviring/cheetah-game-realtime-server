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
	let meta_object_id = helper.create_user_object();
	ffi::client::set_source_object_to_meta(&meta_object_id);
	let object_id_with_structure = helper.create_user_object();
	ffi::command::structure::set_structure(&object_id_with_structure, 1, &BufferFFI::from(vec![1]));
	helper.wait_udp();

	ffi::client::set_current_client(client2);
	ffi::client::receive();

	assert!(matches!(STRUCTURE.lock().unwrap().as_ref(), Some(object_id) if *object_id==meta_object_id))
}

lazy_static! {
	static ref STRUCTURE: Mutex<Option<GameObjectIdFFI>> = Mutex::new(Default::default());
}

extern "C" fn on_structure_listener(
	meta: &S2CMetaCommandInformationFFI,
	_object_id: &GameObjectIdFFI,
	_field_id: FieldId,
	_buffer: &BufferFFI,
) {
	STRUCTURE.lock().unwrap().replace(meta.source_object.clone());
}
