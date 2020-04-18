use bytebuffer::ByteBuffer;
use log::error;

use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor, error_c2s_command, get_field_and_change, trace_c2s_command};
use crate::relay::room::clients::Client;
use crate::relay::room::groups::Access;
use crate::relay::room::objects::ErrorGetObjectWithCheckAccess;
use crate::relay::room::objects::object::{FieldID, ObjectFieldType};
use crate::relay::room::room::{GlobalObjectId, Room};

/// Обновление счетчика
#[derive(Debug)]
pub struct UpdateFloatCounterC2SCommand {
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub increment: f64,
}


impl C2SCommandDecoder for UpdateFloatCounterC2SCommand {
	const COMMAND_ID: u8 = 4;
	
	fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn C2SCommandExecutor>> {
		let global_object_id = bytes.read_u64();
		let counter_id = bytes.read_u16();
		let increment = bytes.read_f64();
		return if global_object_id.is_err() || counter_id.is_err() || increment.is_err() {
			Option::None
		} else {
			Option::Some(Box::new(
				UpdateFloatCounterC2SCommand {
					global_object_id: global_object_id.unwrap(),
					field_id: counter_id.unwrap(),
					increment: increment.unwrap(),
				}
			))
		};
	}
}

impl C2SCommandExecutor for UpdateFloatCounterC2SCommand {
	fn execute(&self, client: &Client, room: &mut Room) {
		trace_c2s_command("UpdateFloatCounter", room, client, format!("params {:?}", self));
		get_field_and_change(
			"UpdateFloatCounter",
			room,
			client,
			self.global_object_id,
			self.field_id,
			ObjectFieldType::FloatCounter,
			|object|
				{
					let value = object.increment_float_counter(self.field_id, self.increment);
					format!("increment done, result {}", value)
				},
		);
	}
}

