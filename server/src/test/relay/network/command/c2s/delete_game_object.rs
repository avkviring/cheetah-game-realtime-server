use bytebuffer::ByteBuffer;

use crate::relay::network::command::c2s::C2SCommandDecoder;
use crate::relay::network::command::c2s::delete_game_object::DeleteGameObjectC2SCommand;

#[test]
fn should_decode() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	
	let result = DeleteGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	
	let result = &*(result.unwrap());
	let command = result.as_any_ref().downcast_ref::<DeleteGameObjectC2SCommand>().unwrap();
	
	assert_eq!(command.global_object_id, 100)
}

#[test]
fn should_not_decode_when_data_not_enough() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u32(100);
	
	let result = DeleteGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}