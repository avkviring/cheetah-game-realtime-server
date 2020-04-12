use std::io::Error;

use bytebuffer::ByteBuffer;

use crate::relay::network::commands::{ClientCommandDecoder, ClientCommandExecutor};
use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;

/// удаление игрового объекта
pub struct DeleteGameObject {
	pub global_object_id: u64
}


impl ClientCommandDecoder for DeleteGameObject {
	const COMMAND_ID: u8 = 2;
	fn decode(bytes: &mut ByteBuffer) -> Option<Box<ClientCommandExecutor>> {
		return match bytes.read_u64() {
			Ok(id) => {
				Option::Some(Box::new(DeleteGameObject { global_object_id: id }))
			}
			Err(_) => Option::None,
		};
	}
}

impl ClientCommandExecutor for DeleteGameObject {
	fn execute(&self, client: &Client, room: &mut Room) {
		room.delete_game_object(client.configuration.id, self.global_object_id);
	}
}

