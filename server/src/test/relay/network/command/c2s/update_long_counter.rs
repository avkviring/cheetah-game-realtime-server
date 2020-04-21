use bytebuffer::ByteBuffer;

use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor};
use crate::relay::network::command::c2s::update_long_counter::UpdateLongCounterC2SCommand;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::room::Room;
use crate::test::relay::room::setup_and_two_client;
use crate::relay::room::objects::object::GameObjectTemplate;

#[test]
fn should_decode() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_i64(10);
	
	let result = UpdateLongCounterC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	
	let result = &*(result.unwrap());
	let command = result.as_any_ref().downcast_ref::<UpdateLongCounterC2SCommand>().unwrap();
	
	assert_eq!(command.global_object_id, 100);
	assert_eq!(command.field_id, 5);
	assert_eq!(command.increment, 10);
}

#[test]
fn should_not_decode_when_data_not_enough_1() {
	let mut buffer = ByteBuffer::new();
	let result = UpdateLongCounterC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_2() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u32(100);
	buffer.write_u32(5);
	let result = UpdateLongCounterC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}


#[test]
fn test_execute_command() {
	let field_id = 10;
	
	let (mut room, client, _) = setup_and_two_client();
	let global_object_id = room.create_client_game_object(
		&client.clone(),
		0,
		&GameObjectTemplate::stub_with_group(0b100000)
	).ok().unwrap();
	
	let command = UpdateLongCounterC2SCommand {
		global_object_id,
		field_id,
		increment: 10,
	};
	command.execute(&client.clone(), &mut room);
	command.execute(&client.clone(), &mut room);
	
	let rc_object = room.objects.get(global_object_id).unwrap().clone();
	let object = (*rc_object).borrow();
	
	assert_eq!(object.get_long_counter(field_id), 20)
}