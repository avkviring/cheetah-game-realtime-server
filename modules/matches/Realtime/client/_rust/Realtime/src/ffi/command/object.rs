use cheetah_matches_realtime_common::commands::binary_value::BinaryValue;
use cheetah_matches_realtime_common::commands::c2s::C2SCommand;
use cheetah_matches_realtime_common::commands::types::create::C2SCreatedGameObjectCommand;
use cheetah_matches_realtime_common::commands::types::delete::DeleteGameObjectCommand;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, BufferFFI, ForwardedCommandFFI, GameObjectIdFFI};

#[no_mangle]
pub extern "C" fn set_create_object_listener(client_id: ClientId, listener: extern "C" fn(&GameObjectIdFFI, template: u16)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_create_object = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn set_created_object_listener(client_id: ClientId, listener: extern "C" fn(&GameObjectIdFFI)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_created_object = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn create_object(client_id: ClientId, template: u16, access_group: u64, result: &mut GameObjectIdFFI) -> u8 {
	execute_with_client(client_id, |client| {
		let game_object_id = client.create_game_object(template, access_group)?;
		*result = game_object_id;
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn created_object(client_id: ClientId, object_id: &GameObjectIdFFI, room_owner: bool, singleton_key: &BufferFFI) -> u8 {
	let singleton_key = if singleton_key.len > 0 {
		Some(BinaryValue::from(singleton_key))
	} else {
		None
	};
	send_command(
		client_id,
		C2SCommand::CreatedGameObject(C2SCreatedGameObjectCommand {
			object_id: From::from(object_id),
			room_owner,
			singleton_key,
		}),
	)
}

#[no_mangle]
pub extern "C" fn set_delete_object_listener(client_id: ClientId, listener: extern "C" fn(&GameObjectIdFFI)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_delete_object = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn delete_object(client_id: ClientId, object_id: &GameObjectIdFFI) -> u8 {
	send_command(
		client_id,
		C2SCommand::Delete(DeleteGameObjectCommand {
			object_id: From::from(object_id),
		}),
	)
}

#[no_mangle]
pub extern "C" fn set_forwarded_command_listener(client_id: ClientId, listener: extern "C" fn(ForwardedCommandFFI)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_forwarded_command = Some(listener);
		Ok(())
	})
}
