use cheetah_relay::network::c2s::decode_end_execute_c2s_commands;
use cheetah_relay_common::network::niobuffer::NioBuffer;

use crate::unit::relay::room::clients::client_stub;
use crate::unit::relay::room::room::room_stub;

#[test]
fn should_decode_result_false_if_empty_buffer() {
    let mut buffer = NioBuffer::new();
    buffer.flip();
    let decode_result = decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
    assert_eq!(decode_result, 0);
}

#[test]
fn should_decode_result_false_if_partial_buffer() {
    // let mut buffer = NioBuffer::new();
    // buffer.write_u8(C2S_TEST_COMMAND);
    // buffer.flip();
    // let decode_result = decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
    // assert_eq!(decode_result, 0);
    // assert_eq!(buffer.read_u8().ok().unwrap(), C2S_TEST_COMMAND)
    unimplemented!()
}


#[test]
fn should_decode() {
    let mut buffer = NioBuffer::new();
    // buffer.write_u8(C2S_TEST_COMMAND).ok().unwrap();
    // buffer.write_u64(100).ok().unwrap();
    // buffer.flip();
    // let decode_result = decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
    // assert_eq!(decode_result, 1);
    unimplemented!()
}

#[test]
fn should_decode_more_one_command() {
    // let mut buffer = NioBuffer::new();
    // buffer.write_u8(C2S_TEST_COMMAND).ok().unwrap();
    // buffer.write_u64(100).ok().unwrap();
    // buffer.write_u8(C2S_TEST_COMMAND).ok().unwrap();
    // buffer.write_u64(200).ok().unwrap();
    // buffer.flip();
    // let decode_result = decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
    // assert_eq!(buffer.has_remaining(), false);
    // assert_eq!(decode_result, 2);
    unimplemented!()
}


