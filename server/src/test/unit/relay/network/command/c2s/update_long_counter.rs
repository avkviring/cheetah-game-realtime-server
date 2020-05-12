use crate::relay::network::command::c2s::update_long_counter::UpdateLongCounterC2SCommand;
use crate::relay::room::objects::object::GameObjectTemplate;
use crate::test::unit::relay::room::setup_and_two_client;
use crate::relay::network::types::niobuffer::NioBuffer;

#[test]
fn should_decode() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100).ok().unwrap();
	buffer.write_u16(5).ok().unwrap();
	buffer.write_i64(10).ok().unwrap();
	buffer.flip();
	
	let result = UpdateLongCounterC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	
	let command = result.unwrap();
	assert_eq!(command.global_object_id, 100);
	assert_eq!(command.field_id, 5);
	assert_eq!(command.increment, 10);
}

#[test]
fn should_not_decode_when_data_not_enough_1() {
	let mut buffer = NioBuffer::new();
	buffer.flip();
	let result = UpdateLongCounterC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_2() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok().unwrap();
	buffer.write_u32(5).ok().unwrap();
	buffer.flip();
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
		&GameObjectTemplate::stub_with_group(0b100000),
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