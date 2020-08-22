use std::borrow::Borrow;

use cheetah_relay::network::c2s::ServerCommandExecutor;
use cheetah_relay_common::network::command::structure::StructureCommand;

use crate::unit::relay::room::setup_and_two_client;
use crate::unit::relay::network::command::create_game_object;

#[test]
fn test_execute_command() {
	let struct_data = vec![1, 2, 3, 4, 5];
	let field_id = 10;
	let (mut room, client, _) = setup_and_two_client();
	let (server_object_id, client_object_id) = create_game_object(&mut room, &client);
	
	let command = StructureCommand {
		object_id: client_object_id,
		field_id,
		structure: struct_data.clone(),
	};
	command.execute(client.borrow(), &mut room);
	
	let rc_object = room.objects.get(&server_object_id).unwrap();
	let object = (*rc_object).borrow();
	let object_struct_data = object.get_struct(field_id).unwrap();
	assert_eq!(object_struct_data, &struct_data)
}