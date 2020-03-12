use bytes::{Buf, Bytes, BytesMut};

use crate::relay::network::commands::{ClientCommandExecutor, CommandDecoder};
use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;

/// команда создания игрового объекта
pub struct CreateGameObject {
    /// локальный идентификатор объекта
    local_id: u32,
    groups: Vec<u8>,
}


impl CommandDecoder for CreateGameObject {
    const COMMAND_ID: u8 = 1;

    fn decode(bytes: &mut Bytes) -> Option<Box<dyn ClientCommandExecutor>> {
        return if bytes.remaining() > 4 + 1 {
            let local_id = bytes.get_u32();
            Option::Some(Box::new(CreateGameObject {
                local_id,
                groups: Default::default(),
            }))
        } else {
            Option::None
        };
    }
}

impl ClientCommandExecutor for CreateGameObject {
    fn execute(&self, client: &Client, room: &mut Room) {
        room.create_client_game_object(client.configuration.id, self.local_id, self.groups.as_ref());
    }
}
