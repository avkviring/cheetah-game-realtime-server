use bytebuffer::ByteBuffer;

use crate::relay::network::command::c2s::C2SCommandDecoder;
use crate::relay::network::command::c2s::event::EventC2SCommand;

#[test]
fn should_decode() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	buffer.write_bytes(&vec![1, 2, 3, 4, 5]);
	
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	
	let result = &*(result.unwrap());
	let command = result.as_any_ref().downcast_ref::<EventC2SCommand>().unwrap();
	
	assert_eq!(command.global_object_id, 100);
	assert_eq!(command.field_id, 5);
	assert_eq!(command.event_data, vec![1, 2, 3, 4, 5]);
}

#[test]
fn should_not_decode_when_data_not_enough_1() {
	let mut buffer = ByteBuffer::new();
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_2() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_3() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_4() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_5() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	buffer.write_u8(1);
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}
