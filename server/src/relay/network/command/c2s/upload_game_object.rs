use std::collections::HashMap;
use bytebuffer::ByteBuffer;
use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor, error_c2s_command, trace_c2s_command};
use crate::relay::room::clients::Client;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::objects::CreateObjectError;
use crate::relay::room::objects::object::{DataStruct, FieldID, FloatCounter, GameObjectTemplate, LongCounter};
use crate::relay::room::room::{LocalObjectId, Room};

/// команда создания игрового объекта
#[derive(Debug)]
pub struct UploadGameObjectC2SCommand {
	pub local_id: LocalObjectId,
	pub template: GameObjectTemplate,
}


impl C2SCommandDecoder for UploadGameObjectC2SCommand {
	const COMMAND_ID: u8 = 1;
	
	fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn C2SCommandExecutor>> {
		if bytes.len() - bytes.get_rpos() < 4 + 8 + 2 + 2 + 2 {
			return Option::None;
		}
		
		let local_id = bytes.read_u32().unwrap();
		let groups = bytes.read_u64().unwrap();
		let long_counter_count = bytes.read_u16().unwrap();
		let float_counter_count = bytes.read_u16().unwrap();
		let structures_counter_count = bytes.read_u16().unwrap();
		
		let size_for_read_counters = (long_counter_count as u64 + float_counter_count as u64) * (2 + 8);
		if bytes.len() - bytes.get_rpos() < size_for_read_counters as usize {
			return Option::None;
		}
		
		let long_counters = UploadGameObjectC2SCommand::read_long_counters(bytes, long_counter_count);
		let float_counters = UploadGameObjectC2SCommand::read_float_counters(bytes, float_counter_count);
		
		let mut structures = HashMap::<FieldID, DataStruct>::new();
		for _ in 0..structures_counter_count {
			let field_id = bytes.read_u16();
			let size = bytes.read_u16();
			if field_id.is_err() || size.is_err() {
				return Option::None;
			}
			
			let size = size.unwrap() as usize;
			if bytes.len() - bytes.get_rpos() < size {
				return Option::None;
			}
			
			let struct_data = bytes.read_bytes(size);
			if struct_data.is_err() {
				return Option::None;
			}
			
			structures.insert(field_id.unwrap(), DataStruct { data: struct_data.unwrap() });
		}
		
		
		return Option::Some(
			Box::new(
				UploadGameObjectC2SCommand {
					local_id,
					template: GameObjectTemplate {
						long_counters,
						float_counters,
						structures,
						groups: AccessGroups::from(groups),
					},
				}));
	}
}


impl C2SCommandExecutor for UploadGameObjectC2SCommand {
	fn execute(&self, client: &Client, room: &mut Room) {
		trace_c2s_command("CreateGameObject", room, client, format!("params {:?}", self));
		let result = room.create_client_game_object(client, self.local_id, &self.template);
		match result {
			Ok(id) => {
				trace_c2s_command("CreateGameObject", room, client, format!("Object created with id {}", id));
			}
			Err(error) => {
				match error {
					CreateObjectError::IncorrectGroups => {
						error_c2s_command("CreateGameObject", room, client, format!("Incorrect access group"));
					}
				}
			}
		}
	}
}

impl UploadGameObjectC2SCommand {
	fn read_long_counters(bytes: &mut ByteBuffer, long_counter_count: u16) -> HashMap<u16, LongCounter> {
		let mut long_counters = HashMap::<FieldID, LongCounter>::new();
		for _ in 0..long_counter_count {
			let counter_id = bytes.read_u16();
			let counter_value = bytes.read_i64();
			long_counters.insert(counter_id.unwrap(), LongCounter { counter: counter_value.unwrap() });
		}
		long_counters
	}
	
	fn read_float_counters(bytes: &mut ByteBuffer, float_counter_count: u16) -> HashMap<u16, FloatCounter> {
		let mut long_counters = HashMap::<FieldID, FloatCounter>::new();
		for _ in 0..float_counter_count {
			let counter_id = bytes.read_u16();
			let counter_value = bytes.read_f64();
			long_counters.insert(counter_id.unwrap(), FloatCounter { counter: counter_value.unwrap() });
		}
		long_counters
	}
}
