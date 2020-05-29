use cheetah_relay::network::c2s::ServerCommandExecutor;
use cheetah_relay_common::network::command::float_counter::IncrementFloatCounterC2SCommand;
use cheetah_relay_common::room::access::AccessGroups;

use crate::unit::relay::room::setup_and_two_client;

#[test]
fn test_execute_command() {
    let field_id = 10;

    let (mut room, client, _) = setup_and_two_client();
    let global_object_id = room.create_client_game_object(
        &client.clone(),
        0,
        AccessGroups::from(0b10_0000),
        Default::default(),
    ).ok().unwrap();

    IncrementFloatCounterC2SCommand {
        global_object_id,
        field_id,
        increment: 10.0,
    }.execute(&client.clone(), &mut room);

    IncrementFloatCounterC2SCommand {
        global_object_id,
        field_id,
        increment: 20.0,
    }.execute(&client.clone(), &mut room);


    let rc_object = room.objects.get(global_object_id).unwrap();
    let object = (*rc_object).borrow();

    assert_eq!(object.get_float_counter(field_id) as u64, 30)
}