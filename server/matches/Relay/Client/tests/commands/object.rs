use std::sync::Mutex;

use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::command::S2CMetaCommandInformationFFI;
use cheetah_matches_relay_client::ffi::{BufferFFI, GameObjectIdFFI};

use crate::helpers::helper::*;
use crate::helpers::server::*;
use cheetah_matches_relay_common::constants::FieldId;

///
/// Тест на создание/удаление объекта
///
#[test]
fn test() {
	let (helper, client1, client2) = setup(IntegrationTestServerBuilder::default());

	ffi::client::set_current_client(client2);
	ffi::command::object::set_create_object_listener(on_object_create);
	ffi::command::structure::set_structure_listener(on_structure_listener);
	ffi::command::object::set_created_object_listener(on_object_created);
	ffi::command::object::set_delete_object_listener(on_object_delete);
	ffi::command::room::attach_to_room();
	helper.wait_udp();

	ffi::client::set_current_client(client1);
	let mut object_id = GameObjectIdFFI::new();
	ffi::command::object::create_object(1, IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0, &mut object_id);
	let structure_field_id = 10;
	let structure_buffer = BufferFFI::from(vec![125]);
	ffi::command::structure::set_structure(&object_id, structure_field_id, &structure_buffer);
	ffi::command::object::created_object(&object_id);
	ffi::command::object::delete_object(&object_id);

	helper.wait_udp();
	ffi::client::set_current_client(client2);
	ffi::client::receive();

	assert!(matches!(CREATE_OBJECT_ID.lock().unwrap().as_ref(),Option::Some(id) if *id==object_id));
	assert!(
		matches!(STRUCTURE.lock().unwrap().as_ref(),Option::Some((field_id, buffer)) if *field_id == structure_field_id && *buffer == structure_buffer )
	);
	assert!(matches!(CREATED_OBJECT_ID.lock().unwrap().as_ref(),Option::Some(id) if *id==object_id));
	assert!(matches!(DELETED_OBJECT_ID.lock().unwrap().as_ref(),Option::Some(id) if *id==object_id));
}

lazy_static! {
	static ref CREATE_OBJECT_ID: Mutex<Option<GameObjectIdFFI>> = Mutex::new(Default::default());
}
lazy_static! {
	static ref CREATED_OBJECT_ID: Mutex<Option<GameObjectIdFFI>> = Mutex::new(Default::default());
}
lazy_static! {
	static ref DELETED_OBJECT_ID: Mutex<Option<GameObjectIdFFI>> = Mutex::new(Default::default());
}

lazy_static! {
	static ref STRUCTURE: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
}

extern "C" fn on_object_create(_: &S2CMetaCommandInformationFFI, object_id: &GameObjectIdFFI, _: u16) {
	CREATE_OBJECT_ID.lock().unwrap().replace((*object_id).clone());
}

extern "C" fn on_object_created(_: &S2CMetaCommandInformationFFI, object_id: &GameObjectIdFFI) {
	CREATED_OBJECT_ID.lock().unwrap().replace((*object_id).clone());
}

extern "C" fn on_object_delete(_: &S2CMetaCommandInformationFFI, object_id: &GameObjectIdFFI) {
	DELETED_OBJECT_ID.lock().unwrap().replace((*object_id).clone());
}
extern "C" fn on_structure_listener(_: &S2CMetaCommandInformationFFI, _object_id: &GameObjectIdFFI, field_id: FieldId, buffer: &BufferFFI) {
	STRUCTURE.lock().unwrap().replace((field_id, (*buffer).clone()));
}
