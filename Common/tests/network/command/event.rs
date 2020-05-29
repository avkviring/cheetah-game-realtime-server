use cheetah_relay_common::network::command::{Decoder, Encoder};
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::niobuffer::NioBuffer;

use crate::network::command::create_buffer_with_capacity;

#[test]
fn should_decode() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	buffer.write_bytes(&vec![1, 2, 3, 4, 5]);
	buffer.flip();
	
	let result = EventCommand::decode(&mut buffer);
	assert_eq!(result.is_ok(), true);
	
	let command = result.unwrap();
	
	assert_eq!(command.global_object_id, 100);
	assert_eq!(command.field_id, 5);
	assert_eq!(command.event, vec![1, 2, 3, 4, 5]);
}

#[test]
fn should_decode_fail_when_data_not_enough_1() {
	let mut buffer = NioBuffer::new();
	buffer.flip();
	let result = EventCommand::decode(&mut buffer);
	assert_eq!(result.is_ok(), false);
}

#[test]
fn should_decode_fail_when_data_not_enough_2() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.flip();
	let result = EventCommand::decode(&mut buffer);
	assert_eq!(result.is_ok(), false);
}

#[test]
fn should_decode_fail_when_data_not_enough_3() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.flip();
	let result = EventCommand::decode(&mut buffer);
	assert_eq!(result.is_ok(), false);
}

#[test]
fn should_decode_fail_when_data_not_enough_4() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	buffer.flip();
	let result = EventCommand::decode(&mut buffer);
	assert_eq!(result.is_ok(), false);
}

#[test]
fn should_decode_fail_when_data_not_enough_5() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	buffer.write_u8(1);
	buffer.flip();
	let result = EventCommand::decode(&mut buffer);
	assert_eq!(result.is_ok(), false);
}

#[test]
fn should_encode_when_buffer_is_enough() {
	let mut buffer = create_buffer_with_capacity(8 + 2 + 2);
	assert_eq!(create_command().encode(&mut buffer).is_ok(), true)
}

#[test]
fn should_encode_fail_when_buffer_for_write_is_small_1() {
	let mut buffer = create_buffer_with_capacity(0);
	assert_eq!(create_command().encode(&mut buffer).is_ok(), false)
}

#[test]
fn should_encode_fail_when_buffer_for_write_is_small_2() {
	let mut buffer = create_buffer_with_capacity(8);
	assert_eq!(create_command().encode(&mut buffer).is_ok(), false)
}

#[test]
fn should_encode_fail_when_buffer_for_write_is_small_3() {
	let mut buffer = create_buffer_with_capacity(8 + 2);
	assert_eq!(create_command().encode(&mut buffer).is_ok(), false)
}

#[test]
fn should_encode_fail_when_buffer_for_write_is_small_4() {
	let mut buffer = create_buffer_with_capacity(8 + 2 + 1);
	assert_eq!(create_command().encode(&mut buffer).is_ok(), false)
}

fn create_command() -> EventCommand {
	EventCommand {
		global_object_id: Default::default(),
		field_id: Default::default(),
		event: Default::default(),
	}
}