use cheetah_relay::network::command::c2s::event::EventC2SCommand;
use cheetah_relay::network::types::niobuffer::NioBuffer;

#[test]
fn should_decode() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	buffer.write_bytes(&vec![1, 2, 3, 4, 5]);
	buffer.flip();
	
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	
	let command = result.unwrap();
	
	assert_eq!(command.global_object_id, 100);
	assert_eq!(command.field_id, 5);
	assert_eq!(command.event_data, vec![1, 2, 3, 4, 5]);
}

#[test]
fn should_not_decode_when_data_not_enough_1() {
	let mut buffer = NioBuffer::new();
	buffer.flip();
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_2() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.flip();
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_3() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.flip();
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_4() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	buffer.flip();
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn should_not_decode_when_data_not_enough_5() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	buffer.write_u8(1);
	buffer.flip();
	let result = EventC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}
