use crate::relay::network::command::c2s::{get_field_and_change, trace_c2s_command};
use crate::relay::network::types::niobuffer::NioBuffer;
use crate::relay::room::clients::Client;
use crate::relay::room::objects::object::{FieldID, ObjectFieldType};
use crate::relay::room::room::{GlobalObjectId, Room};

/// Обновление счетчика
#[derive(Debug)]
pub struct UpdateLongCounterC2SCommand {
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub increment: i64,
}


impl UpdateLongCounterC2SCommand {
	pub const COMMAND_ID: u8 = 3;
	
	pub fn decode(bytes: &mut NioBuffer) -> Option<UpdateLongCounterC2SCommand> {
		let global_object_id = bytes.read_u64();
		let counter_id = bytes.read_u16();
		let increment = bytes.read_i64();
		if global_object_id.is_err() || counter_id.is_err() || increment.is_err() {
			Option::None
		} else {
			Option::Some(
				UpdateLongCounterC2SCommand {
					global_object_id: global_object_id.ok().unwrap(),
					field_id: counter_id.ok().unwrap(),
					increment: increment.ok().unwrap(),
				}
			)
		}
	}
	pub fn execute(&self, client: &Client, room: &mut Room) {
		trace_c2s_command("UpdateLongCounter", room, client, format!("params {:?}", self));
		get_field_and_change(
			"UpdateLongCounter",
			room,
			client,
			self.global_object_id,
			self.field_id,
			ObjectFieldType::LongCounter,
			|room, object| {
				let value = room.object_increment_long_counter(object, self.field_id, self.increment);
				format!("increment done, result {}", value)
			},
		)
	}
}

