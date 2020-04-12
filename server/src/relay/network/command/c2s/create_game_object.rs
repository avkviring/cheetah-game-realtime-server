use std::io::Read;

use bytebuffer::ByteBuffer;

use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;
use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor};

/// команда создания игрового объекта
#[derive(Debug)]
pub struct CreateGameObjectC2SCommand {
	/// локальный идентификатор объекта
	pub local_id: u32,
	pub groups: Vec<u8>,
}


impl C2SCommandDecoder for CreateGameObjectC2SCommand {
	const COMMAND_ID: u8 = 1;
	
	fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn C2SCommandExecutor>> {
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
		
		return Option::Some(
			Box::new(
				CreateGameObjectC2SCommand {
					local_id: local_id.unwrap(),
					groups,
				}));
	}
}

impl C2SCommandExecutor for CreateGameObjectC2SCommand {
	fn execute(&self, client: &Client, room: &mut Room) {
		room.create_client_game_object(client.configuration.id, self.local_id, self.groups.as_ref());
	}
}
