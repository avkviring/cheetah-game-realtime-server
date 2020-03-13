use std::io::Read;

use bytebuffer::ByteBuffer;

use crate::relay::network::commands::{ClientCommandDecoder, ClientCommandExecutor};
use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;

/// команда создания игрового объекта
#[derive(Debug)]
pub struct CreateGameObject {
    /// локальный идентификатор объекта
    pub local_id: u32,
    pub groups: Vec<u8>,
}


impl ClientCommandDecoder for CreateGameObject {
    const COMMAND_ID: u8 = 1;

    fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn ClientCommandExecutor>> {
        let local_id = bytes.read_u32();

        if local_id.is_err() {
            return Option::None;
        }

        let group_count = bytes.read_u8();

        if group_count.is_err() {
            return Option::None;
        }

        let mut groups = vec![];
        for i in 0..group_count.unwrap() {
            let group = bytes.read_u8();
            if group.is_err() {
                return Option::None;
            }
            groups.push(group.unwrap())
        }

        return Option::Some(Box::new(CreateGameObject {
            local_id: local_id.unwrap(),
            groups,
        }));
    }
}

impl ClientCommandExecutor for CreateGameObject {
    fn execute(&self, client: &Client, room: &mut Room) {
        room.create_client_game_object(client.configuration.id, self.local_id, self.groups.as_ref());
    }
}
