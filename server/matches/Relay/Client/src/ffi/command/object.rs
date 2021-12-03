use cheetah_matches_relay_common::commands::command::load::CreatedGameObjectCommand;
use cheetah_matches_relay_common::commands::command::unload::DeleteGameObjectCommand;
use cheetah_matches_relay_common::commands::command::C2SCommand;

use crate::ffi::command::{send_command, S2CMetaCommandInformationFFI};
use crate::ffi::{execute_with_client, GameObjectIdFFI};
use crate::registry::ClientId;

#[no_mangle]
pub extern "C" fn set_create_object_listener(
	client_id: ClientId,
	listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, template: u16),
) -> bool {
	execute_with_client(client_id, |client| client.register_create_object_listener(listener)).is_ok()
}

#[no_mangle]
pub extern "C" fn set_created_object_listener(
	client_id: ClientId,
	listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI),
) -> bool {
	execute_with_client(client_id, |client| client.listener_created_object = Option::Some(listener)).is_ok()
}

#[no_mangle]
pub extern "C" fn create_object(client_id: ClientId, template: u16, access_group: u64, result: &mut GameObjectIdFFI) -> bool {
	execute_with_client(client_id, |client| {
		let game_object_id = client.create_game_object(template, access_group);
		*result = game_object_id;
	})
	.is_ok()
}

#[no_mangle]
pub extern "C" fn created_object(client_id: ClientId, object_id: &GameObjectIdFFI) -> bool {
	send_command(
		client_id,
		C2SCommand::Created(CreatedGameObjectCommand {
			object_id: From::from(object_id),
		}),
	)
}

#[no_mangle]
pub extern "C" fn set_delete_object_listener(
	client_id: ClientId,
	listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI),
) -> bool {
	execute_with_client(client_id, |client| client.register_delete_object_listener(listener)).is_ok()
}

#[no_mangle]
pub extern "C" fn delete_object(client_id: ClientId, object_id: &GameObjectIdFFI) -> bool {
	send_command(
		client_id,
		C2SCommand::Delete(DeleteGameObjectCommand {
			object_id: From::from(object_id),
		}),
	)
}
