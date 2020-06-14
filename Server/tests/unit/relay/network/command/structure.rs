use std::borrow::Borrow;

use cheetah_relay::network::c2s::ServerCommandExecutor;
use cheetah_relay_common::network::command::structure::StructureCommand;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::Owner;

use crate::unit::relay::room::setup_and_two_client;

#[test]
fn test_execute_command() {
	let struct_data = vec![1, 2, 3, 4, 5];
	let field_id = 10;
	let (mut room, client, _) = setup_and_two_client();
	let object_id = GameObjectId::new(0, Owner::Client(client.configuration.id));
	room.new_game_object(
		object_id.clone(),
		AccessGroups::from(0b10_0000),
		Default::default(),
	);
	
	let command = StructureCommand {
		object_id: object_id.clone(),
		field_id,
		structure: struct_data.clone(),
	};
	command.execute(client.borrow(), &mut room);
	
	let rc_object = room.objects.get(&object_id).unwrap().clone();
	let object = (*rc_object).borrow();
	let object_struct_data = object.get_struct(field_id).unwrap();
	assert_eq!(object_struct_data, &struct_data)
}