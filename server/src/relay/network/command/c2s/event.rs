use bytebuffer::ByteBuffer;
use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor, get_field_and_change, trace_c2s_command};
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


impl C2SCommandDecoder for EventC2SCommand {
	const COMMAND_ID: u8 = 6;
	
	fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn C2SCommandExecutor>> {
		let global_object_id = bytes.read_u64();
		let field_id = bytes.read_u16();
		let size = bytes.read_u16();
		return if global_object_id.is_err()
			|| field_id.is_err()
			|| size.is_err()
		{
			Option::None
		} else {
			let bytes = bytes.read_bytes(size.unwrap() as usize);
			if bytes.is_err() {
				Option::None
			} else {
				Option::Some(Box::new(
					EventC2SCommand {
						global_object_id: global_object_id.unwrap(),
						field_id: field_id.unwrap(),
						event_data: bytes.unwrap(),
					}
				))
			}
		};
	}
}

impl C2SCommandExecutor for EventC2SCommand {
	fn execute(&self, client: &Client, room: &mut Room) {
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