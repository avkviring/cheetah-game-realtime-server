use cheetah_client::ffi;
use cheetah_client::ffi::command::BufferFFI;
use cheetah_common::commands::CommandTypeId;
use cheetah_common::room::object::GameObjectId;

use crate::helpers::helper::setup;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

///
/// Тест на создание/удаление объекта
///
#[test]
fn test() {
	let (helper, [client1, client2]) = setup(Default::default());
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();

	let mut object_id = GameObjectId::default();
	ffi::command::object::create_object(client1, 1, IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0, &mut object_id);

	let structure_field_id = 10;
	let structure_buffer = BufferFFI::from(vec![125].as_slice());
	ffi::command::structure::set_structure(client1, &object_id, structure_field_id, &structure_buffer);
	ffi::command::object::created_object(client1, &object_id, false, &Default::default());
	ffi::command::object::delete_object(client1, &object_id);

	let commands = helper.receive(client2);
	assert_eq!(commands[0].command_type, CommandTypeId::CreateGameObject);
	assert_eq!(commands[1].command_type, CommandTypeId::SetStructure);
	assert_eq!(commands[2].command_type, CommandTypeId::CreatedGameObject);
	assert_eq!(commands[3].command_type, CommandTypeId::DeleteObject);
}
