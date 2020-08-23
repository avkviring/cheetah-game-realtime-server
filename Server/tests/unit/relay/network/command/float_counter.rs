use cheetah_relay::network::c2s::ServerCommandExecutor;
use cheetah_relay_common::network::command::float_counter::IncrementFloat64CounterC2SCommand;

use crate::unit::relay::network::command::create_game_object;
use crate::unit::relay::room::setup_and_two_client;

#[test]
fn test_execute_command() {
	let field_id = 10;
	
	let (mut room, client, _) = setup_and_two_client();
	let (server_object_id, client_object_id) = create_game_object(&mut room, &client);
	
	IncrementFloat64CounterC2SCommand {
		object_id: client_object_id.clone(),
		field_id,
		increment: 10.0,
	}.execute(&client, &mut room);
	
	IncrementFloat64CounterC2SCommand {
		object_id: client_object_id,
		field_id,
		increment: 20.0,
	}.execute(&client, &mut room);
	
	
	let rc_object = room.objects.get(&server_object_id).unwrap();
	let object = (*rc_object).borrow();
	
	assert_eq!(object.get_float_counter(field_id) as u64, 30)
}

