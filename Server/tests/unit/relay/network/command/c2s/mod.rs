use cheetah_relay::network::c2s::decode_end_execute_c2s_commands;
use cheetah_relay_common::network::command::{CommandCode, Encoder};
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::upload::UploadGameObjectC2SCommand;
use cheetah_relay_common::network::niobuffer::NioBuffer;

use crate::unit::relay::room::clients::client_stub;
use crate::unit::relay::room::room::room_stub;

#[test]
fn should_decode_result_false_if_empty_buffer() {
    let mut buffer = NioBuffer::new();
    buffer.flip();
    let command_count =
        decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
    assert_eq!(command_count, 0);
}

#[test]
fn should_decode_result_false_if_partial_buffer() {
    let mut buffer = NioBuffer::new();
    buffer.write_u8(EventCommand::COMMAND_CODE).unwrap();
    buffer.flip();
    let command_count =
        decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
    assert_eq!(command_count, 0);
    assert_eq!(buffer.read_u8().ok().unwrap(), EventCommand::COMMAND_CODE)
}

#[test]
fn should_decode() {
    let command = UploadGameObjectC2SCommand {
        local_id: Default::default(),
        access_groups: Default::default(),
        fields: Default::default(),
    };
    let mut buffer = NioBuffer::new();
    buffer.write_u8(UploadGameObjectC2SCommand::COMMAND_CODE).unwrap();
    command.encode(&mut buffer).unwrap();
    buffer.flip();
    let command_count =
        decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
    assert_eq!(command_count, 1);
}

#[test]
fn should_decode_more_one_command() {
    let command = UploadGameObjectC2SCommand {
        local_id: Default::default(),
        access_groups: Default::default(),
        fields: Default::default(),
    };
    let mut buffer = NioBuffer::new();
    buffer.write_u8(UploadGameObjectC2SCommand::COMMAND_CODE).unwrap();
    command.encode(&mut buffer).unwrap();
    buffer.write_u8(UploadGameObjectC2SCommand::COMMAND_CODE).unwrap();
    command.encode(&mut buffer).unwrap();
    buffer.flip();

    let decode_result = decode_end_execute_c2s_commands(&mut buffer, &client_stub(0), &mut room_stub());
    assert_eq!(buffer.has_remaining(), false);
    assert_eq!(decode_result, 2);
}
