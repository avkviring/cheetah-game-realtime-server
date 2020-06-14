use cheetah_relay::network::c2s::ServerCommandExecutor;
use cheetah_relay_common::network::command::long_counter::IncrementLongCounterC2SCommand;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::Owner;

use crate::unit::relay::room::setup_and_two_client;

#[test]
fn test_execute_command() {
	let field_id = 10;
	
	let (mut room, client, _) = setup_and_two_client();
	let object_id = GameObjectId::new(0, Owner::Client(client.configuration.id));
	room.new_game_object(
		object_id.clone(),
		AccessGroups::from(0b10_0000),
		Default::default(),
	);
	IncrementLongCounterC2SCommand {
		object_id: object_id.clone(),
		field_id,
		increment: 10,
	}.execute(&client.clone(), &mut room);
	
	IncrementLongCounterC2SCommand {
		object_id: object_id.clone(),
		field_id,
		increment: 20,
	}.execute(&client.clone(), &mut room);
	
	let rc_object = room.objects.get(&object_id).unwrap().clone();
	let object = (*rc_object).borrow();
	assert_eq!(object.get_long_counter(field_id), 30)
}