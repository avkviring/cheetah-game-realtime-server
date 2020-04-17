use bytebuffer::ByteBuffer;
use log::error;

use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor, error_c2s_command, trace_c2s_command};
use crate::relay::room::clients::Client;
use crate::relay::room::groups::Access;
use crate::relay::room::objects::ErrorGetObjectWithCheckAccess;
use crate::relay::room::objects::object::ObjectFieldType;
use crate::relay::room::room::Room;

/// Обновление счетчика
pub struct UpdateLongCounterC2SCommand {
	pub global_object_id: u64,
	pub counter_id: u16,
	pub increment: i64,
}


impl C2SCommandDecoder for UpdateLongCounterC2SCommand {
	const COMMAND_ID: u8 = 3;
	
	fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn C2SCommandExecutor>> {
		let global_object_id = bytes.read_u64();
		let counter_id = bytes.read_u16();
		let increment = bytes.read_i64();
		return if global_object_id.is_err() || counter_id.is_err() || increment.is_err() {
			Option::None
		} else {
			Option::Some(Box::new(
				UpdateLongCounterC2SCommand {
					global_object_id: global_object_id.unwrap(),
					counter_id: counter_id.unwrap(),
					increment: increment.unwrap(),
				}
			))
		};
	}
}

impl C2SCommandExecutor for UpdateLongCounterC2SCommand {
	fn execute(&self, client: &Client, room: &mut Room) {
		trace_c2s_command("UpdateLongCounter", room, client, format!("params {:?}", self));
		
		let result_check = room
			.get_object_with_check_field_access(
				Access::WRITE,
				client,
				self.global_object_id,
				ObjectFieldType::LongCounter,
				self.counter_id);
		
		match result_check {
			Ok(object) => {
				object.increment_counter(self.counter_id, self.increment);
				trace_c2s_command("UpdateLongCounter", room, client, format!("increment done, result {}", object.get_counter(self.counter_id)));
			}
			Err(error) => {
				match error {
					ErrorGetObjectWithCheckAccess::ObjectNotFound => {
						error_c2s_command("UpdateLongCounter", room, client, format!("object not found {}", self.counter_id));
					}
					ErrorGetObjectWithCheckAccess::AccessNotAllowed => {
						error_c2s_command("UpdateLongCounter", room, client, format!("client has not write access to objects {} field {} type {:?}", self.global_object_id, self.counter_id, ObjectFieldType::LongCounter));
					}
				}
			}
		}
	}
}

