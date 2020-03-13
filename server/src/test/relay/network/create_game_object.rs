use std::any::Any;
use std::borrow::Borrow;

use bytebuffer::ByteBuffer;
use traitcast::Traitcast;

use crate::relay::network::commands::{ClientCommandDecoder, ClientCommandExecutor};
use crate::relay::network::commands::create_game_object::CreateGameObject;
use crate::relay::room::clients::Client;

#[test]
fn should_decode_create_game_object() {
    let mut buffer = ByteBuffer::new();
    buffer.write_u32(100);
    buffer.write_u8(2);
    buffer.write_u8(10);
    buffer.write_u8(20);

    let result = CreateGameObject::decode(&mut buffer);
    assert_eq!(result.is_some(), true);

    let result = &*(result.unwrap());
    let command = result.as_any_ref().downcast_ref::<CreateGameObject>().unwrap();

    assert_eq!(command.local_id, 100);
    assert_eq!(command.groups, vec![10 as u8, 20 as u8])
}

#[test]
fn should_not_decode_create_game_object_when_data_not_enough() {
    let mut buffer = ByteBuffer::new();
    buffer.write_u32(100);
    buffer.write_u8(2);
    buffer.write_u8(10);

    let result = CreateGameObject::decode(&mut buffer);
    assert_eq!(result.is_some(), false);
}