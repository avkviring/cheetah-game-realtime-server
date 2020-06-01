use cheetah_relay::network::c2s::ServerCommandExecutor;
use cheetah_relay_common::network::command::long_counter::IncrementLongCounterC2SCommand;
use cheetah_relay_common::room::access::AccessGroups;

use crate::unit::relay::room::setup_and_two_client;

#[test]
fn test_execute_command() {
    let field_id = 10;

    let (mut room, client, _) = setup_and_two_client();
    let global_object_id = room.create_client_game_object(
        &client.clone(),
        0,
        AccessGroups::from(0b100000),
        Default::default(),
    ).ok().unwrap();
    IncrementLongCounterC2SCommand {
        global_object_id,
        field_id,
        increment: 10,
    }.execute(&client.clone(), &mut room);

    IncrementLongCounterC2SCommand {
        global_object_id,
        field_id,
        increment: 20,
    }.execute(&client.clone(), &mut room);

    let rc_object = room.objects.get(global_object_id).unwrap().clone();
    let object = (*rc_object).borrow();
    assert_eq!(object.get_long_counter(field_id), 30)
}