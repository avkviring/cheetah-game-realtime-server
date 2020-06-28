use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::network::command::{CommandCode, Decoder};
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::{IncrementFloatCounterC2SCommand, SetFloatCounterCommand};
use cheetah_relay_common::network::command::long_counter::{IncrementLongCounterC2SCommand, SetLongCounterCommand};
use cheetah_relay_common::network::command::structure::StructureCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::network::command::upload::UploadGameObjectCommand;
use cheetah_relay_common::network::niobuffer::NioBuffer;
use cheetah_relay_common::network::tcp::connection::OnReadBufferError;
use cheetah_relay_common::room::object::ClientGameObjectId;

use crate::room::clients::Client;
use crate::room::objects::ErrorGetObjectWithCheckAccess;
use crate::room::objects::object::GameObject;
use crate::room::Room;

pub mod unload;
pub mod event;
pub mod float_counter;
pub mod long_counter;
pub mod structure;
pub mod upload;

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
		SetLongCounterCommand::COMMAND_CODE => {
			SetLongCounterCommand::decode(buffer)
				.map(|f| f.execute(client, room))
				.map_err(OnReadBufferError::NioBufferError)
		}
		IncrementFloatCounterC2SCommand::COMMAND_CODE => {
			IncrementFloatCounterC2SCommand::decode(buffer)
				.map(|f| f.execute(client, room))
				.map_err(OnReadBufferError::NioBufferError)
		}
		SetFloatCounterCommand::COMMAND_CODE => {
			SetFloatCounterCommand::decode(buffer)
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
	object_id: &ClientGameObjectId,
	action: F,
) where
	F: FnOnce(&mut Room, &mut GameObject) -> String,
{
	let result_check = room.get_object_with_check_field_access(
		client,
		&object_id,
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
		},
	}
}
