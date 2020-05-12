use crate::relay::network::command::c2s::delete_game_object::DeleteGameObjectC2SCommand;
use crate::relay::network::command::c2s::event::EventC2SCommand;
use crate::relay::network::command::c2s::update_float_counter::UpdateFloatCounterC2SCommand;
use crate::relay::network::command::c2s::update_long_counter::UpdateLongCounterC2SCommand;
use crate::relay::network::command::c2s::update_struct::UpdateStructC2SCommand;
use crate::relay::network::command::c2s::upload_game_object::UploadGameObjectC2SCommand;
use crate::relay::network::types::niobuffer::NioBuffer;
use crate::relay::room::clients::Client;
use crate::relay::room::groups::Access;
use crate::relay::room::objects::ErrorGetObjectWithCheckAccess;
use crate::relay::room::objects::object::{FieldID, GameObject, ObjectFieldType};
use crate::relay::room::room::{GlobalObjectId, Room};

pub mod upload_game_object;
pub mod delete_game_object;
pub mod update_long_counter;
pub mod update_float_counter;
pub mod update_struct;
pub mod event;


///
/// Декодирование и выполнение C2S команд
/// return - количество декодированных команд
///
pub fn decode_end_execute_c2s_commands(buffer: &mut NioBuffer, client: &Client, room: &mut Room) -> usize {
	let mut command_count = 0;
	while buffer.has_remaining() {
		buffer.mark();
		let command_code = buffer.read_u8().ok().unwrap();
		let command_decoded = match command_code {
			UploadGameObjectC2SCommand::COMMAND_ID => {
				UploadGameObjectC2SCommand::decode(buffer).map(|f| f.execute(client, room))
			}
			DeleteGameObjectC2SCommand::COMMAND_ID => {
				DeleteGameObjectC2SCommand::decode(buffer).map(|f| f.execute(client, room))
			}
			UpdateLongCounterC2SCommand::COMMAND_ID => {
				UpdateLongCounterC2SCommand::decode(buffer).map(|f| f.execute(client, room))
			}
			UpdateFloatCounterC2SCommand::COMMAND_ID => {
				UpdateFloatCounterC2SCommand::decode(buffer).map(|f| f.execute(client, room))
			}
			UpdateStructC2SCommand::COMMAND_ID => {
				UpdateStructC2SCommand::decode(buffer).map(|f| f.execute(client, room))
			}
			EventC2SCommand::COMMAND_ID => {
				EventC2SCommand::decode(buffer).map(|f| f.execute(client, room))
			}
			C2S_TEST_COMMAND => {
				buffer.read_u64().map(|_| {}).ok()
			}
			_ => {
				log::error!("decoder: unknown command type {}", command_code);
				Option::None
			}
		};
		
		if command_decoded.is_some() {
			command_count += 1;
		} else {
			buffer.reset().ok();
			break;
		}
	};
	command_count
}

pub const C2S_TEST_COMMAND: u8 = 255;


pub fn trace_c2s_command(command: &str, room: &Room, client: &Client, message: String) {
	log::trace!("C2S {:<10} : room {} : client {} : {}", command, room.hash, client.configuration.hash, message);
}

pub fn error_c2s_command(command: &str, room: &Room, client: &Client, message: String) {
	log::error!("C2S {:<10} : room {} : client {} : {}", command, room.hash, client.configuration.hash, message);
}

pub fn get_field_and_change<F>(command_name: &str,
							   room: &mut Room,
							   client: &Client,
							   global_object_id: GlobalObjectId,
							   field_id: FieldID,
							   object_field_type: ObjectFieldType,
							   action: F,
) where F: FnOnce(&mut Room, &mut GameObject) -> String {
	let result_check = room
		.get_object_with_check_field_access(
			Access::WRITE,
			client,
			global_object_id,
			object_field_type,
			field_id);
	
	match result_check {
		Ok(object) => {
			let message = action(room, &mut *(*(object.clone())).borrow_mut());
			trace_c2s_command(command_name, room, client, message)
		}
		Err(error) => {
			match error {
				ErrorGetObjectWithCheckAccess::ObjectNotFound => {
					error_c2s_command(command_name, room, client, format!("object not found {}", global_object_id));
				}
				ErrorGetObjectWithCheckAccess::AccessNotAllowed => {
					error_c2s_command(command_name, room, client, format!("client has not write access to objects {} field {}", global_object_id, field_id));
				}
			}
		}
	}
}