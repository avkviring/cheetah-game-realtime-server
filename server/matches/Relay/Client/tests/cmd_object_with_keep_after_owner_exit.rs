use std::sync::Mutex;

use lazy_static::lazy_static;

use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::GameObjectIdFFI;

use crate::helpers::helper::setup;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

///
/// Объекты с флагом keep не должны удаляться после выхода владельца
///
#[test]
fn test() {
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());
	ffi::command::object::set_delete_object_listener(client2, on_object_delete);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();
	let mut object_id = GameObjectIdFFI::default();
	ffi::command::object::create_object(
		client1,
		1,
		IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0,
		true,
		&mut object_id,
	);
	ffi::command::object::created_object(client1, &object_id);
	helper.wait_udp();
	ffi::client::destroy_client(client1);
	helper.wait_udp();
	ffi::client::receive(client2);

	assert!(DELETED_OBJECT_ID.lock().as_ref().unwrap().is_none());
}

lazy_static! {
	static ref DELETED_OBJECT_ID: Mutex<Option<GameObjectIdFFI>> = Mutex::new(Default::default());
}

extern "C" fn on_object_delete(object_id: &GameObjectIdFFI) {
	DELETED_OBJECT_ID.lock().unwrap().replace((*object_id).clone());
}
