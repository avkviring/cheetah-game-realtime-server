use cheetah_relay::network::c2s::ServerCommandExecutor;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::Owner;

use crate::unit::relay::room::setup_and_two_client;

#[test]
fn test_execute_command() {
	let (mut room, client, _) = setup_and_two_client();
	let object_id = GameObjectId::new(0, Owner::Client(client.configuration.id));
	room.new_game_object(
		object_id.clone(),
		AccessGroups::from(0b10_0000),
		Default::default(),
	);
	
	let command = UnloadGameObjectCommand {
		object_id: object_id.clone(),
	};
	command.execute(&client.clone(), &mut room);
	assert_eq!(room.objects.get(&object_id).is_none(), true);
}