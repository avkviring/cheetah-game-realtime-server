use crate::relay::network::command::c2s::update_float_counter::UpdateFloatCounterC2SCommand;
use crate::relay::network::types::niobuffer::NioBuffer;
use crate::relay::room::objects::object::GameObjectTemplate;
use crate::test::unit::relay::room::setup_and_two_client;

#[test]
fn should_decode() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100).ok().unwrap();
	buffer.write_u16(5).ok().unwrap();
	buffer.write_f64(10.0).ok().unwrap();
	buffer.flip();
	
	let result = UpdateFloatCounterC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	let command = result.unwrap();
	
	assert_eq!(command.global_object_id, 100);
	assert_eq!(command.field_id, 5);
	assert_eq!(command.increment, 10.0 as f64);
}

#[test]
fn should_not_decode_when_data_not_enough() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok().unwrap();
	buffer.flip();
	let result = UpdateFloatCounterC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn test_execute_command() {
	let field_id = 10;
	
	let (mut room, client, _) = setup_and_two_client();
	let global_object_id = room.create_client_game_object(
		&client.clone(),
		0,
		&GameObjectTemplate::stub_with_group(0b10_0000),
	).ok().unwrap();
	
	let command = UpdateFloatCounterC2SCommand {
		global_object_id,
		field_id,
		increment: 10.0,
	};
	command.execute(&client.clone(), &mut room);
	command.execute(&client, &mut room);
	
	let rc_object = room.objects.get(global_object_id).unwrap();
	let object = (*rc_object).borrow();
	
	assert_eq!(object.get_float_counter(field_id), 20.0 as f64)
}