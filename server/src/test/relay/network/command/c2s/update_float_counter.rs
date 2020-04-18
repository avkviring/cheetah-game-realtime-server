use bytebuffer::ByteBuffer;

use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor};
use crate::relay::network::command::c2s::update_float_counter::UpdateFloatCounterC2SCommand;
use crate::test::relay::room::setup_and_two_client;

#[test]
fn should_decode() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_f64(10.0);
	
	let result = UpdateFloatCounterC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	
	let result = &*(result.unwrap());
	let command = result.as_any_ref().downcast_ref::<UpdateFloatCounterC2SCommand>().unwrap();
	
	assert_eq!(command.global_object_id, 100);
	assert_eq!(command.field_id, 5);
	assert_eq!(command.increment, 10.0);
}

#[test]
fn should_not_decode_when_data_not_enough() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u32(100);
	let result = UpdateFloatCounterC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn test_execute_command() {
	let field_id = 10;
	
	let (mut room, client, _) = setup_and_two_client();
	let global_object_id = room.create_client_game_object(&client.clone(), 0, Option::None).ok().unwrap();
	
	let command = UpdateFloatCounterC2SCommand {
		global_object_id,
		field_id,
		increment: 10.0,
	};
	command.execute(&client.clone(), &mut room);
	command.execute(&client.clone(), &mut room);
	
	let rc_object = room.objects.get(global_object_id).unwrap().clone();
	let object = (*rc_object).borrow();
	
	assert_eq!(object.get_float_counter(field_id), 20.0)
}