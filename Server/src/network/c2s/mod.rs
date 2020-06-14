use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::network::command::{CommandCode, Decoder};
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::IncrementFloatCounterC2SCommand;
use cheetah_relay_common::network::command::long_counter::IncrementLongCounterC2SCommand;
use cheetah_relay_common::network::command::structure::StructureCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::network::command::upload::UploadGameObjectCommand;
use cheetah_relay_common::network::niobuffer::NioBuffer;
use cheetah_relay_common::network::tcp::connection::OnReadBufferError;
use cheetah_relay_common::room::access::Access;
use cheetah_relay_common::room::object::GameObjectId;

use crate::room::clients::Client;
use crate::room::objects::ErrorGetObjectWithCheckAccess;
use crate::room::objects::object::{GameObject, ObjectFieldType};
use crate::room::Room;

///
/// Выполнение серверной команды
///
pub trait ServerCommandExecutor {
	fn execute(self, client: &Client, room: &mut Room);
}

///
/// Декодирование и выполнение C2S команд
/// return - количество декодированных команд
///
pub fn decode_end_execute_c2s_commands(
	buffer: &mut NioBuffer,
	client: &Client,
	room: &mut Room,
) -> Result<(), OnReadBufferError> {
	let command_code = buffer.read_u8().map_err(OnReadBufferError::NioBufferError)?;
	match command_code {
		UploadGameObjectCommand::COMMAND_CODE => {
			UploadGameObjectCommand::decode(buffer)
				.map(|f| f.execute(client, room))
				.map_err(OnReadBufferError::NioBufferError)
		}
		UnloadGameObjectCommand::COMMAND_CODE => {
			UnloadGameObjectCommand::decode(buffer)
				.map(|f| f.execute(client, room))
				.map_err(OnReadBufferError::NioBufferError)
		}
		IncrementLongCounterC2SCommand::COMMAND_CODE => {
			IncrementLongCounterC2SCommand::decode(buffer)
				.map(|f| f.execute(client, room))
				.map_err(OnReadBufferError::NioBufferError)
		}
		IncrementFloatCounterC2SCommand::COMMAND_CODE => {
			IncrementFloatCounterC2SCommand::decode(buffer)
				.map(|f| f.execute(client, room))
				.map_err(OnReadBufferError::NioBufferError)
		}
		StructureCommand::COMMAND_CODE => {
			StructureCommand::decode(buffer)
				.map(|f| f.execute(client, room))
				.map_err(OnReadBufferError::NioBufferError)
		}
		EventCommand::COMMAND_CODE => {
			EventCommand::decode(buffer)
				.map(|f| f.execute(client, room))
				.map_err(OnReadBufferError::NioBufferError)
		}
		code => {
			Result::Err(OnReadBufferError::UnknownCommand(code))
		}
	}
}

pub fn trace_c2s_command(command: &str, room: &Room, client: &Client, message: String) {
	log::trace!(
        "C2S {:<10} : room {} : client {} : {}",
        command,
        room.hash,
        client.configuration.hash,
        message
    );
}

pub fn error_c2s_command(command: &str, room: &Room, client: &Client, message: String) {
	log::error!(
        "C2S {:<10} : room {} : client {} : {}",
        command,
        room.hash,
        client.configuration.hash,
        message
    );
}

pub fn get_field_and_change<F>(
	command_name: &str,
	room: &mut Room,
	client: &Client,
	object_id: &GameObjectId,
	field_id: FieldID,
	object_field_type: ObjectFieldType,
	action: F,
) where
	F: FnOnce(&mut Room, &mut GameObject) -> String,
{
	let result_check = room.get_object_with_check_field_access(
		Access::WRITE,
		client,
		&object_id,
		object_field_type,
		field_id,
	);
	
	match result_check {
		Ok(object) => {
			let message = action(room, &mut *(*(object.clone())).borrow_mut());
			trace_c2s_command(command_name, room, client, message)
		}
		Err(error) => match error {
			ErrorGetObjectWithCheckAccess::ObjectNotFound => {
				error_c2s_command(
					command_name,
					room,
					client,
					format!("object not found {:?}", &object_id),
				);
			}
			ErrorGetObjectWithCheckAccess::AccessNotAllowed => {
				error_c2s_command(
					command_name,
					room,
					client,
					format!("client has not write access to objects {:?} field {}", &object_id, field_id),
				);
			}
		},
	}
}
