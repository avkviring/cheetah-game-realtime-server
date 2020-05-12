use crate::relay::network::command::c2s::{get_field_and_change, trace_c2s_command};
use crate::relay::network::types::niobuffer::NioBuffer;
use crate::relay::room::clients::Client;
use crate::relay::room::objects::object::{FieldID, ObjectFieldType};
use crate::relay::room::room::{GlobalObjectId, Room};

/// Обновление структуры
#[derive(Debug)]
pub struct UpdateStructC2SCommand {
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub data: Vec<u8>,
}


impl UpdateStructC2SCommand {
	pub const COMMAND_ID: u8 = 5;
	
	pub fn decode(buffer: &mut NioBuffer) -> Option<UpdateStructC2SCommand> {
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
			if buffer.remaining() >= size {
				let field_id = field_id.unwrap();
				let global_object_id = global_object_id.unwrap();
				let data = buffer.read_to_vec(size).unwrap();
				let command = UpdateStructC2SCommand {
					global_object_id,
					field_id,
					data,
				};
				Option::Some(command)
			} else {
				Option::None
			}
		}
	}
	
	pub fn execute(&self, client: &Client, room: &mut Room) {
		trace_c2s_command("UpdateStruct", room, client, format!("params {:?}", self));
		get_field_and_change(
			"UpdateStruct",
			room,
			client,
			self.global_object_id,
			self.field_id,
			ObjectFieldType::Struct,
			|room, object|
				{
					room.object_update_struct(object, self.field_id, &self.data);
					format!("update struct done")
				},
		);
	}
}