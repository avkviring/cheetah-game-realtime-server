use bytebuffer::ByteBuffer;

use crate::relay::network::command::c2s::C2SCommandDecoder;
use crate::relay::network::command::c2s::update_struct::UpdateStructC2SCommand;

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
fn should_not_decode_when_data_not_enough() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	buffer.write_u16(5);
	buffer.write_u16(5);
	let result = UpdateStructC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}