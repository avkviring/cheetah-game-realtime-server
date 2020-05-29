use std::borrow::Borrow;
use std::collections::HashMap;

use cheetah_relay::network::c2s::ServerCommandExecutor;
use cheetah_relay_common::network::command::upload::UploadGameObjectC2SCommand;

use crate::unit::relay::room::setup_and_two_client;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay::room::objects::object::GameObject;

#[test]
fn test_execute_command() {
    let (mut room, client, _) = setup_and_two_client();
    let mut long_counters = HashMap::new();
    long_counters.insert(10, 20 );

    let mut float_counters = HashMap::new();
    float_counters.insert(20,  30.0 );

    let mut structures = HashMap::new();
    structures.insert(50, vec![0, 1, 2, 3, 4] );

    let command = UploadGameObjectC2SCommand {
        local_id: 155,
        access_groups: AccessGroups::from(0b10_0000),
        fields: GameObjectFields {
            long_counters,
            float_counters,
            structures,
        },
    };
    command.execute(&client.clone(), &mut room);
    let global_object_id = GameObject::get_global_object_id_by_client(client.borrow(), 155);

    let rc_object = room.objects.get(global_object_id).unwrap();
    let game_object = (*rc_object).borrow();
    assert_eq!(game_object.access_groups.contains_group(5), true);
    assert_eq!(*game_object.fields.long_counters.get(&10).unwrap(), 20);
    assert_eq!(*game_object.fields.float_counters.get(&20).unwrap(), 30.0 as f64);
    assert_eq!(*game_object.fields.structures.get(&50).unwrap(), vec![0, 1, 2, 3, 4]);
}