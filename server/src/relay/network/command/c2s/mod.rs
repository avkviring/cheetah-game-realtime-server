/// Команды с клиента
use bytebuffer::ByteBuffer;
use traitcast::TraitcastFrom;

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

/// Декодер входящей команды
pub trait C2SCommandDecoder {
	/// идентификатор команды
	const COMMAND_ID: u8;
	
	/// Декодирование команды
	/// bytes - массив байт, из которого будет прочитана информация
	/// если результат Option::None то указатель данных в bytes будет сброшен в начало
	fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn C2SCommandExecutor>>;
}


/// Интерфейс команды с клиента
pub trait C2SCommandExecutor: TraitcastFrom {
	/// Выполнить команду
	fn execute(&self, client: &Client, room: &mut Room);
}


pub fn trace_c2s_command(command: &str, room: &Room, client: &Client, message: String) {
	log::trace!("C2S: {:<10} : room {} : client {} : message {}", command, room.id, client.configuration.hash, message);
}

pub fn error_c2s_command(command: &str, room: &Room, client: &Client, message: String) {
	log::trace!("C2S: {:<10} : room {} : client {} : message {}", command, room.id, client.configuration.hash, message);
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