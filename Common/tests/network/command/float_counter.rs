use cheetah_relay_common::network::command::{Decoder, Encoder};
use cheetah_relay_common::network::command::float_counter::{IncrementFloatCounterC2SCommand, SetFloatCounterCommand};
use cheetah_relay_common::network::niobuffer::NioBuffer;

use crate::network::command::create_buffer_with_capacity;

#[test]
fn should_decode() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100).ok().unwrap();
	buffer.write_u16(5).ok().unwrap();
	buffer.write_f64(10.0).ok().unwrap();
	buffer.flip();
	
	let result = IncrementFloatCounterC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_ok(), true);
	let command = result.unwrap();
	
	assert_eq!(command.global_object_id, 100);
	assert_eq!(command.field_id, 5);
	assert_eq!(command.increment as u64, 10);
}

#[test]
fn should_decode_fail_when_data_not_enough() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100).ok().unwrap();
	buffer.flip();
	let result = IncrementFloatCounterC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_ok(), false);
}

#[test]
fn should_encode_when_buffer_is_enough() {
	let mut buffer = create_buffer_with_capacity(8 + 2 + 8);
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
	let mut buffer = create_buffer_with_capacity(8 + 2 + 7);
	assert_eq!(create_command().encode(&mut buffer).is_ok(), false)
}

fn create_command() -> SetFloatCounterCommand {
	SetFloatCounterCommand {
		global_object_id: Default::default(),
		field_id: Default::default(),
		value: Default::default(),
	}
}