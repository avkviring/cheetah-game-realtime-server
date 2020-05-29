use cheetah_relay_common::network::command::{Decoder, Encoder};
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::network::niobuffer::NioBuffer;

use crate::network::command::create_buffer_with_capacity;

#[test]
fn should_decode() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.flip();
	let result = UnloadGameObjectCommand::decode(&mut buffer);
	assert_eq!(result.is_ok(), true);
	let command = result.unwrap();
	
	assert_eq!(command.global_object_id, 100)
}

#[test]
fn should_not_decode_when_data_not_enough() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100);
	buffer.flip();
	let result = UnloadGameObjectCommand::decode(&mut buffer);
	assert_eq!(result.is_ok(), false);
}


#[test]
fn should_true_when_buffer_is_enough() {
	let mut buffer = create_buffer_with_capacity(8);
	assert_eq!(create_command().encode(&mut buffer).is_ok(), true)
}

#[test]
fn should_false_when_buffer_for_write_is_small() {
	let mut buffer = create_buffer_with_capacity(7);
	assert_eq!(create_command().encode(&mut buffer).is_ok(), false)
}

fn create_command() -> UnloadGameObjectCommand {
	UnloadGameObjectCommand {
		global_object_id: Default::default(),
	}
}