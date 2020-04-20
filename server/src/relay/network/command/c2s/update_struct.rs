use bytebuffer::ByteBuffer;
use log::error;

use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor, error_c2s_command, get_field_and_change, get_field_and_change2, trace_c2s_command};
use crate::relay::room::clients::Client;
use crate::relay::room::groups::Access;
use crate::relay::room::objects::ErrorGetObjectWithCheckAccess;
use crate::relay::room::objects::object::{FieldID, GameObject, ObjectFieldType};
use crate::relay::room::room::{GlobalObjectId, Room};

/// Обновление структуры
#[derive(Debug)]
pub struct UpdateStructC2SCommand {
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub struct_data: Vec<u8>,
}


impl C2SCommandDecoder for UpdateStructC2SCommand {
	const COMMAND_ID: u8 = 5;
	
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
					UpdateStructC2SCommand {
						global_object_id: global_object_id.unwrap(),
						field_id: field_id.unwrap(),
						struct_data: bytes.unwrap(),
					}
				))
			}
		};
	}
}

impl C2SCommandExecutor for UpdateStructC2SCommand {
	fn execute(&self, client: &Client, room: &mut Room) {
		trace_c2s_command("UpdateStruct", room, client, format!("params {:?}", self));
		get_field_and_change2(
			"UpdateStruct",
			room,
			client,
			self.global_object_id,
			self.field_id,
			ObjectFieldType::Struct,
			|room, object|
				{
					room.object_update_struct(object, self.field_id, &self.struct_data);
					format!("update struct done")
				},
		);
	}
}