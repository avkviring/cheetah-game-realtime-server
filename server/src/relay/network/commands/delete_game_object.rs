use bytes::Bytes;

use crate::relay::network::commands::{ClientCommandExecutor, CommandDecoder};
use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;

/// удаление игрового объекта
pub struct DeleteGameObject {
    local_id: u32
}


impl CommandDecoder for DeleteGameObject {
    const COMMAND_ID: u8 = 2;

    fn decode(bytes: &mut Bytes) -> Option<Box<ClientCommandExecutor>> {
        unimplemented!()
    }
}

impl ClientCommandExecutor for DeleteGameObject {
    fn execute(&self, client: &Client, room: &mut Room) {
        unimplemented!()
    }
}

