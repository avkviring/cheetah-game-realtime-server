use cheetah_relay::network::c2s::ServerCommandExecutor;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;

use crate::unit::relay::room::setup_and_two_client;

#[test]
fn test_execute_command() {
    let (mut room, client, _) = setup_and_two_client();
    let global_object_id = room.create_client_game_object(
        &client.clone(),
        0,
        AccessGroups::from(0b100000),
        GameObjectFields::default(),
    ).ok().unwrap();

    let command = UnloadGameObjectCommand {
        global_object_id,
    };
    command.execute(&client.clone(), &mut room);
    assert_eq!(room.objects.get(global_object_id).is_none(), true);
}