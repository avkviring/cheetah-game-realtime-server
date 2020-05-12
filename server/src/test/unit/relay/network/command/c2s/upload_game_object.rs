use std::borrow::Borrow;
use std::collections::HashMap;

use crate::relay::network::command::c2s::upload_game_object::UploadGameObjectC2SCommand;
use crate::relay::network::types::niobuffer::NioBuffer;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::objects::object::{DataStruct, FloatCounter, GameObject, GameObjectTemplate, LongCounter};
use crate::test::unit::relay::room::setup_and_two_client;

#[test]
fn should_decode() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0b110).ok(); // groups
	buffer.write_u16(1).ok(); // long counter count
	buffer.write_u16(2).ok(); // float counter count
	buffer.write_u16(1).ok(); // struct counter count
	buffer.write_u16(10).ok(); // long counter field id
	buffer.write_i64(55).ok(); // long counter value
	buffer.write_u16(15).ok(); // float counter field id
	buffer.write_f64(15.0).ok(); // float counter value
	buffer.write_u16(17).ok(); // float counter field id
	buffer.write_f64(19.0).ok(); // float counter value
	buffer.write_u16(5).ok(); // struct field id
	buffer.write_u16(10).ok(); // struct size id
	buffer.write_bytes(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]).ok(); // field data
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	
	
	let command = result.unwrap();
	let template = &command.template;
	assert_eq!(command.local_id, 100);
	assert_eq!(template.groups.contains_group(0), false);
	assert_eq!(template.groups.contains_group(1), true);
	assert_eq!(template.groups.contains_group(2), true);
	assert_eq!(template.groups.contains_group(3), false);
	
	assert_eq!(template.long_counters.len(), 1);
	assert_eq!(template.long_counters.get(&10).unwrap().counter, 55);
	
	assert_eq!(template.float_counters.len(), 2);
	assert_eq!(template.float_counters.get(&15).unwrap().counter, 15.0 as f64);
	assert_eq!(template.float_counters.get(&17).unwrap().counter, 19.0 as f64);
	
	assert_eq!(template.structures.len(), 1);
	assert_eq!(template.structures.get(&5).unwrap().data, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn should_not_decode_when_data_not_enough_1() {
	let mut buffer = NioBuffer::new();
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_2() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_3() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0).ok(); // groups
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_4() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0).ok(); // groups
	buffer.write_u32(1).ok(); // long counter count
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_5() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0).ok(); // groups
	buffer.write_u16(1).ok(); // long counter count
	buffer.write_u16(2).ok(); // float counter count
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_6() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0).ok(); // groups
	buffer.write_u16(1).ok(); // long counter count
	buffer.write_u16(2).ok(); // float counter count
	buffer.write_u16(1).ok(); // struct counter count
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_7() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0).ok(); // groups
	buffer.write_u16(1).ok(); // long counter count
	buffer.write_u16(2).ok(); // float counter count
	buffer.write_u16(1).ok(); // struct counter count
	buffer.write_u16(10).ok(); // long counter field id
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_8() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0).ok(); // groups
	buffer.write_u16(1).ok(); // long counter count
	buffer.write_u16(2).ok(); // float counter count
	buffer.write_u16(1).ok(); // struct counter count
	buffer.write_u16(10).ok(); // long counter field id
	buffer.write_i64(55).ok(); // long counter value
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_9() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0).ok(); // groups
	buffer.write_u16(1).ok(); // long counter count
	buffer.write_u16(2).ok(); // float counter count
	buffer.write_u16(1).ok(); // struct counter count
	buffer.write_u16(10).ok(); // long counter field id
	buffer.write_i64(55).ok(); // long counter value
	buffer.write_u16(15).ok(); // float counter field id
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_10() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0).ok(); // groups
	buffer.write_u16(1).ok(); // long counter count
	buffer.write_u16(2).ok(); // float counter count
	buffer.write_u16(1).ok(); // struct counter count
	buffer.write_u16(10).ok(); // long counter field id
	buffer.write_i64(55).ok(); // long counter value
	buffer.write_u16(15).ok(); // float counter field id
	buffer.write_f64(15.0).ok(); // float counter value
	buffer.write_u16(17).ok(); // float counter field id
	buffer.write_f64(19.0).ok(); // float counter value
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}


#[test]
fn should_not_decode_when_data_not_enough_11() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0).ok(); // groups
	buffer.write_u16(1).ok(); // long counter count
	buffer.write_u16(2).ok(); // float counter count
	buffer.write_u16(1).ok(); // struct counter count
	buffer.write_u16(10).ok(); // long counter field id
	buffer.write_i64(55).ok(); // long counter value
	buffer.write_u16(15).ok(); // float counter field id
	buffer.write_f64(15.0).ok(); // float counter value
	buffer.write_u16(17).ok(); // float counter field id
	buffer.write_f64(19.0).ok(); // float counter value
	buffer.write_u16(5).ok(); // struct field id
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_12() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0).ok(); // groups
	buffer.write_u16(1).ok(); // long counter count
	buffer.write_u16(2).ok(); // float counter count
	buffer.write_u16(1).ok(); // struct counter count
	buffer.write_u16(10).ok(); // long counter field id
	buffer.write_i64(55).ok(); // long counter value
	buffer.write_u16(15).ok(); // float counter field id
	buffer.write_f64(15.0).ok(); // float counter value
	buffer.write_u16(17).ok(); // float counter field id
	buffer.write_f64(19.0).ok(); // float counter value
	buffer.write_u16(5).ok(); // struct field id
	buffer.write_u16(10).ok(); // struct size id
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_13() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok(); // local_object_id
	buffer.write_u64(0).ok(); // groups
	buffer.write_u16(1).ok(); // long counter count
	buffer.write_u16(2).ok(); // float counter count
	buffer.write_u16(1).ok(); // struct counter count
	buffer.write_u16(10).ok(); // long counter field id
	buffer.write_i64(55).ok(); // long counter value
	buffer.write_u16(15).ok(); // float counter field id
	buffer.write_f64(15.0).ok(); // float counter value
	buffer.write_u16(17).ok(); // float counter field id
	buffer.write_f64(19.0).ok(); // float counter value
	buffer.write_u16(5).ok(); // struct field id
	buffer.write_u16(10).ok(); // struct size id
	buffer.write_bytes(&[0, 1, 2, 3, 4, 5, 6, 7]).ok(); // field data
	buffer.flip();
	let result = UploadGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}


#[test]
fn test_execute_command() {
	let (mut room, client, _) = setup_and_two_client();
	let mut long_counters = HashMap::new();
	long_counters.insert(10, LongCounter { counter: 20 });
	
	let mut float_counters = HashMap::new();
	float_counters.insert(20, FloatCounter { counter: 30.0 });
	
	let mut structures = HashMap::new();
	structures.insert(50, DataStruct { data: vec![0, 1, 2, 3, 4] });
	
	let command = UploadGameObjectC2SCommand {
		local_id: 155,
		template: GameObjectTemplate {
			long_counters,
			float_counters,
			structures,
			groups: AccessGroups::from(0b10_0000),
		},
	};
	command.execute(&client.clone(), &mut room);
	let global_object_id = GameObject::get_global_object_id_by_client(client.borrow(), 155);
	
	let rc_object = room.objects.get(global_object_id).unwrap();
	let game_object = (*rc_object).borrow();
	assert_eq!(game_object.groups.contains_group(5), true);
	assert_eq!(game_object.long_counters.get(&10).unwrap().counter, 20);
	assert_eq!(game_object.float_counters.get(&20).unwrap().counter, 30.0 as f64);
	assert_eq!(game_object.structures.get(&50).unwrap().data, vec![0, 1, 2, 3, 4]);
}