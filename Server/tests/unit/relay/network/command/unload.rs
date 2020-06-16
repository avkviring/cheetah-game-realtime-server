use cheetah_relay::network::c2s::ServerCommandExecutor;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;

use crate::unit::relay::network::command::create_game_object;
use crate::unit::relay::room::setup_and_two_client;

#[test]
fn test_execute_command() {
	let (mut room, client, _) = setup_and_two_client();
	let (server_object_id, client_object_id) = create_game_object(&mut room, &client);
	
	let command = UnloadGameObjectCommand {
		object_id: client_object_id,
	};
	command.execute(&client.clone(), &mut room);
	assert_eq!(room.objects.get(&server_object_id).is_none(), true);
}