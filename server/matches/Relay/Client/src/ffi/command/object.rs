use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::commands::types::load::CreatedGameObjectCommand;
use cheetah_matches_relay_common::commands::types::unload::DeleteGameObjectCommand;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, GameObjectIdFFI};

#[no_mangle]
pub extern "C" fn set_create_object_listener(
	client_id: ClientId,
	listener: extern "C" fn(&GameObjectIdFFI, template: u16),
) -> u8 {
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
pub extern "C" fn created_object(client_id: ClientId, object_id: &GameObjectIdFFI) -> u8 {
	send_command(
		client_id,
		C2SCommand::Created(CreatedGameObjectCommand {
			object_id: From::from(object_id),
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
