use std::borrow::Borrow;

use bytebuffer::ByteBuffer;

use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor};
use crate::relay::network::command::c2s::update_struct::UpdateStructC2SCommand;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::room::Room;

#[test]
fn should_decode() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	buffer.write_bytes(&vec![1, 2, 3, 4, 5]);
	
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	
	let result = &*(result.unwrap());
	let command = result.as_any_ref().downcast_ref::<UpdateStructC2SCommand>().unwrap();
	
	assert_eq!(command.global_object_id, 100);
	assert_eq!(command.field_id, 5);
	assert_eq!(command.struct_data, vec![1, 2, 3, 4, 5]);
}

#[test]
fn should_not_decode_when_data_not_enough_1() {
	let mut buffer = ByteBuffer::new();
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_2() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_3() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_4() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_5() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	buffer.write_u8(1);
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn test_execute_command() {
	let struct_data = vec![1, 2, 3, 4, 5];
	let field_id = 10;
	
	let mut room = Room::new();
	room.add_client_to_waiting_list("HASH".to_string(), AccessGroups::new());
	let client = room.client_connect("HASH".to_string()).ok().unwrap();
	let global_object_id = room.create_client_game_object(client.borrow(), 0, Option::None).ok().unwrap();
	
	let command = UpdateStructC2SCommand {
		global_object_id,
		field_id,
		struct_data: struct_data.clone(),
	};
	command.execute(client.borrow(), &mut room);
	
	let rc_object = room.objects.get(global_object_id).unwrap().clone();
	let object = (*rc_object).borrow();
	let object_struct_data = object.get_struct(field_id).unwrap();
	assert_eq!(object_struct_data, &struct_data)
}