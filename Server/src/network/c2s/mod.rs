use std::rc::Rc;


use cheetah_relay_common::network::command::{CommandCode, Decoder};
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::{IncrementFloat64CounterC2SCommand, SetFloat64CounterCommand};
use cheetah_relay_common::network::command::long_counter::{IncrementLongCounterC2SCommand, SetLongCounterCommand};
use cheetah_relay_common::network::command::structure::StructureCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::network::command::upload::UploadGameObjectCommand;
use cheetah_relay_common::network::niobuffer::NioBuffer;
use cheetah_relay_common::network::tcp::connection::OnReadBufferError;
use cheetah_relay_common::room::object::ClientGameObjectId;

use crate::room::clients::Client;
use crate::room::listener::RoomListener;
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
	client: Rc<Client>,
	room: &mut Room,
) -> Result<(), OnReadBufferError> {
	room.listener.set_current_client(client.clone());
	let client = &client;
	let command_code = buffer.read_u8().map_err(OnReadBufferError::NioBufferError)?;
	let result = match command_code {
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
		IncrementFloat64CounterC2SCommand::COMMAND_CODE => {
			IncrementFloat64CounterC2SCommand::decode(buffer)
				.map(|f| f.execute(client, room))
				.map_err(OnReadBufferError::NioBufferError)
		}
		SetFloat64CounterCommand::COMMAND_CODE => {
			SetFloat64CounterCommand::decode(buffer)
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
	};
	room.listener.unset_current_client();
	result
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
			let message = action(room, &mut object.borrow_mut());
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
