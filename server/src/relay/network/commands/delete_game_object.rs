use bytebuffer::ByteBuffer;

use crate::relay::network::commands::{ClientCommandDecoder, ClientCommandExecutor};
use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;

/// удаление игрового объекта
pub struct DeleteGameObject {
    local_id: u32
}


impl ClientCommandDecoder for DeleteGameObject {
    const COMMAND_ID: u8 = 2;

    fn decode(bytes: &mut ByteBuffer) -> Option<Box<ClientCommandExecutor>> {
        unimplemented!()
    }
}

impl ClientCommandExecutor for DeleteGameObject {
    fn execute(&self, client: &Client, room: &mut Room) {
        unimplemented!()
    }
}

