use bytebuffer::ByteBuffer;

use crate::relay::network::commands::ClientCommandDecoder;
use crate::relay::network::commands::delete_game_object::DeleteGameObject;

#[test]
fn should_decode_delete_game_object() {
    let mut buffer = ByteBuffer::new();
    buffer.write_u64(100);

    let result = DeleteGameObject::decode(&mut buffer);
    assert_eq!(result.is_some(), true);

    let result = &*(result.unwrap());
    let command = result.as_any_ref().downcast_ref::<DeleteGameObject>().unwrap();

    assert_eq!(command.global_object_id, 100)
}

#[test]
fn should_not_decode_delete_game_object_when_data_not_enough() {
    let mut buffer = ByteBuffer::new();
    buffer.write_u32(100);

    let result = DeleteGameObject::decode(&mut buffer);
    assert_eq!(result.is_some(), false);
}