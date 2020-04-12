use std::io::Error;

use bytebuffer::ByteBuffer;

use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;
use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor};

/// удаление игрового объекта
pub struct DeleteGameObjectC2SCommand {
	pub global_object_id: u64
}


impl C2SCommandDecoder for DeleteGameObjectC2SCommand {
	const COMMAND_ID: u8 = 2;
	fn decode(bytes: &mut ByteBuffer) -> Option<Box<C2SCommandExecutor>> {
		return match bytes.read_u64() {
			Ok(id) => {
				Option::Some(Box::new(DeleteGameObjectC2SCommand { global_object_id: id }))
			}
			Err(_) => Option::None,
		};
	}
}

impl C2SCommandExecutor for DeleteGameObjectC2SCommand {
	fn execute(&self, client: &Client, room: &mut Room) {
		room.delete_game_object(client.configuration.id, self.global_object_id);
	}
}

