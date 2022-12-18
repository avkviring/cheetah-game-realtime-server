use cheetah_common::commands::binary_value::BinaryValue;
use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::types::create::C2SCreatedGameObjectCommand;
use cheetah_common::commands::types::delete::DeleteGameObjectCommand;
use cheetah_common::room::object::GameObjectId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
use crate::ffi::{execute_with_client, BufferFFI, ForwardedCommandFFI};

#[no_mangle]
pub extern "C" fn set_create_object_listener(client_id: ClientId, listener: extern "C" fn(&GameObjectId, template: u16)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_create_object = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn set_created_object_listener(client_id: ClientId, listener: extern "C" fn(&GameObjectId)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_created_object = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn create_object(client_id: ClientId, template: u16, access_group: u64, result: &mut GameObjectId) -> u8 {
	execute_with_client(client_id, |client| {
		let game_object_id = client.create_game_object(template, access_group)?;
		*result = game_object_id;
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn created_object(client_id: ClientId, object_id: &GameObjectId, room_owner: bool, singleton_key: &BufferFFI) -> u8 {
	let singleton_key = (singleton_key.len > 0).then(|| BinaryValue::from(singleton_key));
	send_command(
		client_id,
		C2SCommand::CreatedGameObject(C2SCreatedGameObjectCommand {
			object_id: *object_id,
			room_owner,
			singleton_key,
		}),
	)
}

#[no_mangle]
pub extern "C" fn set_delete_object_listener(client_id: ClientId, listener: extern "C" fn(&GameObjectId)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_delete_object = Some(listener);
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn delete_object(client_id: ClientId, object_id: &GameObjectId) -> u8 {
	send_command(client_id, C2SCommand::Delete(DeleteGameObjectCommand { object_id: *object_id }))
}

#[no_mangle]
pub extern "C" fn set_forwarded_command_listener(client_id: ClientId, listener: extern "C" fn(ForwardedCommandFFI)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_forwarded_command = Some(listener);
		Ok(())
	})
}
