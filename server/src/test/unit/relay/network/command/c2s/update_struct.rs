use std::borrow::Borrow;

use crate::relay::network::command::c2s::update_struct::UpdateStructC2SCommand;
use crate::relay::network::types::niobuffer::NioBuffer;
use crate::relay::room::objects::object::GameObjectTemplate;
use crate::test::unit::relay::room::setup_and_two_client;

#[test]
fn should_decode() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100).ok().unwrap();
	buffer.write_u16(5).ok().unwrap();
	buffer.write_u16(5).ok().unwrap();
	buffer.write_bytes(&vec![1, 2, 3, 4, 5]).ok().unwrap();
	buffer.flip();
	
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	let command = result.unwrap();
	
	assert_eq!(command.global_object_id, 100);
	assert_eq!(command.field_id, 5);
	assert_eq!(command.data, vec![1, 2, 3, 4, 5]);
}

#[test]
fn should_not_decode_when_data_not_enough_1() {
	let mut buffer = NioBuffer::new();
	buffer.flip();
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_2() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100).ok().unwrap();
	buffer.flip();
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_3() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100).ok().unwrap();
	buffer.write_u16(5).ok().unwrap();
	buffer.flip();
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_4() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100).ok().unwrap();
	buffer.write_u16(5).ok().unwrap();
	buffer.write_u16(5).ok().unwrap();
	buffer.flip();
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_5() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100).ok().unwrap();
	buffer.write_u16(5).ok().unwrap();
	buffer.write_u16(5).ok().unwrap();
	buffer.write_u8(1).ok().unwrap();
	buffer.flip();
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn test_execute_command() {
	let struct_data = vec![1, 2, 3, 4, 5];
	let field_id = 10;
	
	let (mut room, client, _) = setup_and_two_client();
	let global_object_id = room.create_client_game_object(
		&client.clone(),
		0,
		&GameObjectTemplate::stub_with_group(0b100000),
	).ok().unwrap();
	
	let command = UpdateStructC2SCommand {
		global_object_id,
		field_id,
		data: struct_data.clone(),
	};
	command.execute(client.borrow(), &mut room);
	
	let rc_object = room.objects.get(global_object_id).unwrap().clone();
	let object = (*rc_object).borrow();
	let object_struct_data = object.get_struct(field_id).unwrap();
	assert_eq!(object_struct_data, &struct_data)
}