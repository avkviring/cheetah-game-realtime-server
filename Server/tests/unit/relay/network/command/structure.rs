use std::borrow::Borrow;

use cheetah_relay::network::c2s::ServerCommandExecutor;
use cheetah_relay_common::network::command::structure::StructureCommand;
use cheetah_relay_common::room::access::AccessGroups;

use crate::unit::relay::room::setup_and_two_client;

#[test]
fn test_execute_command() {
    let struct_data = vec![1, 2, 3, 4, 5];
    let field_id = 10;

    let (mut room, client, _) = setup_and_two_client();
    let global_object_id = room.create_client_game_object(
        &client.clone(),
        0,
        AccessGroups::from(0b100000),
        Default::default(),
    ).ok().unwrap();

    let command = StructureCommand {
        global_object_id,
        field_id,
        structure: struct_data.clone(),
    };
    command.execute(client.borrow(), &mut room);

    let rc_object = room.objects.get(global_object_id).unwrap().clone();
    let object = (*rc_object).borrow();
    let object_struct_data = object.get_struct(field_id).unwrap();
    assert_eq!(object_struct_data, &struct_data)
}