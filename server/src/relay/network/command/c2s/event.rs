use crate::relay::network::command::c2s::{get_field_and_change, trace_c2s_command};
use crate::relay::network::types::niobuffer::NioBuffer;
use crate::relay::room::clients::Client;
use crate::relay::room::objects::object::{FieldID, ObjectFieldType};
use crate::relay::room::room::{GlobalObjectId, Room};

/// Событие по объекту
#[derive(Debug)]
pub struct EventC2SCommand {
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub event_data: Vec<u8>,
}


impl EventC2SCommand {
	pub const COMMAND_ID: u8 = 6;
	
	pub fn decode(buffer: &mut NioBuffer) -> Option<EventC2SCommand> {
		let global_object_id = buffer.read_u64();
		let field_id = buffer.read_u16();
		let size = buffer.read_u16();
		if global_object_id.is_err()
			|| field_id.is_err()
			|| size.is_err()
		{
			Option::None
		} else {
			let size = size.unwrap() as usize;
			if size <= buffer.remaining() {
				let global_object_id = global_object_id.unwrap();
				let field_id = field_id.unwrap();
				let event_data = buffer.read_to_vec(size).unwrap();
				let command = EventC2SCommand {
					global_object_id,
					field_id,
					event_data,
				};
				
				Option::Some(command)
			} else {
				Option::None
			}
		}
	}
	pub fn execute(&self, client: &Client, room: &mut Room) {
		trace_c2s_command("Event", room, client, format!("params {:?}", self));
		get_field_and_change(
			"Event",
			room,
			client,
			self.global_object_id,
			self.field_id,
			ObjectFieldType::Event,
			|room, object|
				{
					room.object_send_event(object, self.field_id, &self.event_data);
					format!("send event {} done", self.field_id)
				},
		);
	}
}