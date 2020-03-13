use std::rc::Rc;

use bytebuffer::ByteBuffer;

use crate::relay::network::commands::{ClientCommandExecutor, ClientCommandDecoder};
use crate::relay::network::commands::create_game_object::CreateGameObject;
use crate::relay::network::decoder::Decoder;
use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;

#[test]
fn should_decode_result_false_if_empty_buffer() {
    let (mut decoder, mut buffer, _) = setup();
    let decode_result = decoder.decode(&mut buffer);
    assert_eq!(decode_result, false);
}

#[test]
fn should_decode_result_false_if_partial_buffer() {
    let (mut decoder, mut buffer, command_id) = setup();
    buffer.write_u8(command_id);
    let decode_result = decoder.decode(&mut buffer);
    assert_eq!(decode_result, false);
    assert_eq!(buffer.read_u8().unwrap(), command_id)
}


#[test]
fn should_decode() {
    let (mut decoder, mut buffer, command_id) = setup();
    buffer.write_u8(command_id);
    buffer.write_u32(100);
    let decode_result = decoder.decode(&mut buffer);
    assert_eq!(decode_result, true);
}

#[test]
fn should_decode_more_one_command() {
    let (mut decoder, mut buffer, command_id) = setup();
    buffer.write_u8(command_id);
    buffer.write_u32(100);
    buffer.write_u8(command_id);
    buffer.write_u32(200);
    let decode_result = decoder.decode(&mut buffer);
    assert_eq!(buffer.get_rpos(), buffer.get_wpos());
    assert_eq!(decode_result, true);
}


fn setup() -> (Decoder, ByteBuffer, u8) {
    let mut decoder = Decoder::new(Rc::new(Client::stub()));
    let command_id = 55;
    decoder.add_decoder(command_id, decode);
    (decoder, ByteBuffer::new(), command_id)
}

fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn ClientCommandExecutor>> {
    bytes
        .read_u32()
        .map(|f| Box::new(SomeCommand {}) as Box<dyn ClientCommandExecutor>)
        .ok()
}

struct SomeCommand {}

impl ClientCommandExecutor for SomeCommand {
    fn execute(&self, client: &Client, room: &mut Room) {
        unimplemented!()
    }
}