use std::sync::Mutex;

use lazy_static::lazy_static;

use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::{BufferFFI, GameObjectIdFFI};
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::helpers::helper::setup;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

///
/// Тест на создание/удаление комнатного объекта
///
#[test]
fn should_create_room_object() {
	let (helper, [client1, client2, client3]) = setup(IntegrationTestServerBuilder::default());


	ffi::command::object::set_create_object_listener(client1, on_object_create_client1);
	ffi::command::structure::set_structure_listener(client1, on_structure_listener_client1);
	ffi::command::object::set_created_object_listener(client1, on_object_created_client1);

	ffi::command::object::set_create_object_listener(client2, on_object_create_client2);
	ffi::command::structure::set_structure_listener(client2, on_structure_listener_client2);
	ffi::command::object::set_created_object_listener(client2, on_object_created_client2);

	ffi::command::room::attach_to_room(client1);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();

	let mut object_id = GameObjectIdFFI::default();
	ffi::command::object::create_room_object(
		client1,
		1,
		IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0,
		&mut object_id,
	);

	let structure_field_id = 10;
	let structure_buffer = BufferFFI::from(vec![125]);
	ffi::command::structure::set_structure(client1, &object_id, structure_field_id, &structure_buffer);
	ffi::command::object::created_object(client1, &object_id);

	helper.wait_udp();
	ffi::client::receive(client1);
	ffi::client::receive(client2);

	// обект должен загрузиться на создавшего его пользователя
	assert!(
		matches!(CREATE_OBJECT_ID_CLIENT1.lock().unwrap().as_ref(),Option::Some(id) if id
		.room_owner)
	);
	assert!(
		matches!(STRUCTURE_CLIENT1.lock().unwrap().as_ref(),Option::Some((field_id, buffer)) if
			*field_id == structure_field_id && *buffer == structure_buffer )
	);
	assert!(
		matches!(CREATED_OBJECT_ID_CLIENT1.lock().unwrap().as_ref(),Option::Some(id) if id
		.room_owner)
	);

	// объект должен загрузиться на другого пользователя
	assert!(matches!(CREATE_OBJECT_ID_CLIENT2.lock().unwrap().as_ref(),Option::Some(id) if id.room_owner));
	assert!(
		matches!(STRUCTURE_CLIENT2.lock().unwrap().as_ref(),Option::Some((field_id, buffer)) if *field_id == structure_field_id && *buffer == structure_buffer )
	);
	assert!(matches!(CREATED_OBJECT_ID_CLIENT2.lock().unwrap().as_ref(),Option::Some(id) if id.room_owner));

	// третий клиент заходит в комнату после создания объекта
	ffi::command::object::set_create_object_listener(client3, on_object_create_client3);
	ffi::command::structure::set_structure_listener(client3, on_structure_listener_client3);
	ffi::command::object::set_created_object_listener(client3, on_object_created_client3);
	ffi::command::room::attach_to_room(client3);
	helper.wait_udp();
	ffi::client::receive(client3);

	assert!(
		matches!(CREATE_OBJECT_ID_CLIENT3.lock().unwrap().as_ref(),Option::Some(id) if id
		.room_owner)
	);
	assert!(
		matches!(STRUCTURE_CLIENT3.lock().unwrap().as_ref(),Option::Some((field_id, buffer)) if 
			*field_id == structure_field_id && *buffer == structure_buffer )
	);
	assert!(
		matches!(CREATED_OBJECT_ID_CLIENT3.lock().unwrap().as_ref(),Option::Some(id) if id
		.room_owner)
	);
}

lazy_static! {
	static ref CREATE_OBJECT_ID_CLIENT1: Mutex<Option<GameObjectIdFFI>> = Mutex::new(Default::default());
	static ref CREATED_OBJECT_ID_CLIENT1: Mutex<Option<GameObjectIdFFI>> = Mutex::new(Default::default());
	static ref STRUCTURE_CLIENT1: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
	static ref CREATE_OBJECT_ID_CLIENT2: Mutex<Option<GameObjectIdFFI>> = Mutex::new(Default::default());
	static ref CREATED_OBJECT_ID_CLIENT2: Mutex<Option<GameObjectIdFFI>> = Mutex::new(Default::default());
	static ref STRUCTURE_CLIENT2: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
	static ref CREATE_OBJECT_ID_CLIENT3: Mutex<Option<GameObjectIdFFI>> = Mutex::new(Default::default());
	static ref CREATED_OBJECT_ID_CLIENT3: Mutex<Option<GameObjectIdFFI>> = Mutex::new(Default::default());
	static ref STRUCTURE_CLIENT3: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
}

extern "C" fn on_object_create_client1(object_id: &GameObjectIdFFI, _: u16) {
	CREATE_OBJECT_ID_CLIENT1.lock().unwrap().replace((*object_id).clone());
}

extern "C" fn on_object_created_client1(object_id: &GameObjectIdFFI) {
	CREATED_OBJECT_ID_CLIENT1.lock().unwrap().replace((*object_id).clone());
}

extern "C" fn on_structure_listener_client1(
	_: RoomMemberId,
	_object_id: &GameObjectIdFFI,
	field_id: FieldId,
	buffer: &BufferFFI,
) {
	STRUCTURE_CLIENT1.lock().unwrap().replace((field_id, (*buffer).clone()));
}

extern "C" fn on_object_create_client2(object_id: &GameObjectIdFFI, _: u16) {
	CREATE_OBJECT_ID_CLIENT2.lock().unwrap().replace((*object_id).clone());
}

extern "C" fn on_object_created_client2(object_id: &GameObjectIdFFI) {
	CREATED_OBJECT_ID_CLIENT2.lock().unwrap().replace((*object_id).clone());
}

extern "C" fn on_structure_listener_client2(
	_: RoomMemberId,
	_object_id: &GameObjectIdFFI,
	field_id: FieldId,
	buffer: &BufferFFI,
) {
	STRUCTURE_CLIENT2.lock().unwrap().replace((field_id, (*buffer).clone()));
}

extern "C" fn on_object_create_client3(object_id: &GameObjectIdFFI, _: u16) {
	CREATE_OBJECT_ID_CLIENT3.lock().unwrap().replace((*object_id).clone());
}

extern "C" fn on_object_created_client3(object_id: &GameObjectIdFFI) {
	CREATED_OBJECT_ID_CLIENT3.lock().unwrap().replace((*object_id).clone());
}

extern "C" fn on_structure_listener_client3(
	_: RoomMemberId,
	_object_id: &GameObjectIdFFI,
	field_id: FieldId,
	buffer: &BufferFFI,
) {
	STRUCTURE_CLIENT3.lock().unwrap().replace((field_id, (*buffer).clone()));
}
