use cheetah_relay::network::command::c2s::{C2S_TEST_COMMAND, decode_end_execute_c2s_commands};
use cheetah_relay::network::types::niobuffer::NioBuffer;
use cheetah_relay::room::clients::Client;
use cheetah_relay::room::room::Room;
use crate::unit::relay::room::clients::client_stub;
use crate::unit::relay::room::room::room_stub;

pub mod upload_game_object;
pub mod delete_game_object;
pub mod update_long_counter;
pub mod update_float_counter;
pub mod update_struct;
pub mod event;

#[test]
fn should_decode_result_false_if_empty_buffer() {
	let mut buffer = NioBuffer::new();
	buffer.flip();
	let decode_result = decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
	assert_eq!(decode_result, 0);
}

#[test]
fn should_decode_result_false_if_partial_buffer() {
	let mut buffer = NioBuffer::new();
	buffer.write_u8(C2S_TEST_COMMAND);
	buffer.flip();
	let decode_result = decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
	assert_eq!(decode_result, 0);
	assert_eq!(buffer.read_u8().ok().unwrap(), C2S_TEST_COMMAND)
}


#[test]
fn should_decode() {
	let mut buffer = NioBuffer::new();
	buffer.write_u8(C2S_TEST_COMMAND).ok().unwrap();
	buffer.write_u64(100).ok().unwrap();
	buffer.flip();
	let decode_result = decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
	assert_eq!(decode_result, 1);
}

#[test]
fn should_decode_more_one_command() {
	let mut buffer = NioBuffer::new();
	buffer.write_u8(C2S_TEST_COMMAND).ok().unwrap();
	buffer.write_u64(100).ok().unwrap();
	buffer.write_u8(C2S_TEST_COMMAND).ok().unwrap();
	buffer.write_u64(200).ok().unwrap();
	buffer.flip();
	let decode_result = decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
	assert_eq!(buffer.has_remaining(), false);
	assert_eq!(decode_result, 2);
}


