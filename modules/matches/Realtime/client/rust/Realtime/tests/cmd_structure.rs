use std::collections::HashMap;
use std::sync::Mutex;

use cheetah_matches_realtime::room::template::config::Permission;
use cheetah_matches_realtime_client::ffi;
use cheetah_matches_realtime_client::ffi::{BufferFFI, GameObjectIdFFI};
use cheetah_matches_realtime_common::commands::field::FieldId;
use cheetah_matches_realtime_common::commands::FieldType;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::helpers::helper::setup;
use crate::helpers::server::IntegrationTestServerBuilder;

use lazy_static::lazy_static;

pub mod helpers;

#[test]
fn should_set() {
	let (helper, [client1, client2]) = setup(Default::default());

	ffi::command::structure::set_structure_listener(client2, on_structure_listener);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();

	let object_id = helper.create_member_object(client1);
	let structure_buffer = BufferFFI::from(vec![100]);
	let structure_field_id = 10;
	ffi::command::structure::set_structure(client1, &object_id, structure_field_id, &structure_buffer);

	helper.wait_udp();
	ffi::client::receive(client2);

	assert!(matches!(
		STRUCTURE.lock().unwrap().as_ref(),
		Option::Some((field_id, buffer))
			if *field_id == structure_field_id && *buffer == structure_buffer
	));
}

#[test]
fn should_compare_and_set() {
	let mut builder = IntegrationTestServerBuilder::default();

	let field_id_with_reset = 1;
	let field_id = 2;

	// устанавливаем RW для получения команды с сервера на клиента источника команды
	builder.set_permission(
		IntegrationTestServerBuilder::DEFAULT_TEMPLATE,
		field_id_with_reset,
		FieldType::Structure,
		IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
		Permission::Rw,
	);

	builder.set_permission(
		IntegrationTestServerBuilder::DEFAULT_TEMPLATE,
		field_id,
		FieldType::Structure,
		IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
		Permission::Rw,
	);
	let (helper, [client1, client2]) = setup(builder);

	ffi::command::structure::set_structure_listener(client1, on_compare_and_set_listener);
	ffi::command::room::attach_to_room(client1);
	let object_id = helper.create_member_object(client1);
	helper.wait_udp();

	ffi::command::room::attach_to_room(client2);
	// проверяем, что установится только первое значение
	ffi::command::structure::compare_and_set_structure(
		client2,
		&object_id,
		field_id_with_reset,
		&vec![0].into(),
		&vec![100].into(),
		true,
		&vec![42].into(),
	);
	ffi::command::structure::compare_and_set_structure(client2, &object_id, field_id, &vec![0].into(), &vec![200].into(), false, &vec![0].into());
	ffi::command::structure::compare_and_set_structure(
		client2,
		&object_id,
		field_id_with_reset,
		&vec![0].into(),
		&vec![200].into(),
		true,
		&vec![24].into(),
	);
	helper.wait_udp();

	ffi::client::receive(client1);
	assert_eq!(*COMPARE_AND_SET.lock().unwrap().get(&field_id_with_reset).unwrap(), vec![100].into());
	assert_eq!(*COMPARE_AND_SET.lock().unwrap().get(&field_id).unwrap(), vec![200].into());

	// теперь второй клиент разрывает соединение
	// первый наблюдает за тем что значение поменяется на reset
	ffi::client::destroy_client(client2);
	helper.wait_udp();

	ffi::client::receive(client1);
	assert_eq!(*COMPARE_AND_SET.lock().unwrap().get(&field_id_with_reset).unwrap(), vec![42].into());
}

lazy_static! {
	static ref STRUCTURE: Mutex<Option<(FieldId, BufferFFI)>> = Mutex::new(Default::default());
}

lazy_static! {
	static ref COMPARE_AND_SET: Mutex<HashMap<FieldId, BufferFFI>> = Mutex::new(Default::default());
}

extern "C" fn on_structure_listener(_: RoomMemberId, _object_id: &GameObjectIdFFI, field_id: FieldId, buffer: &BufferFFI) {
	STRUCTURE.lock().unwrap().replace((field_id, (*buffer).clone()));
}

extern "C" fn on_compare_and_set_listener(_: RoomMemberId, _object_id: &GameObjectIdFFI, field_id: FieldId, value: &BufferFFI) {
	COMPARE_AND_SET.lock().unwrap().insert(field_id, value.to_owned());
}
