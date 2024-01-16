use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::types::create::C2SCreatedGameObject;
use cheetah_common::room::buffer::Buffer;
use cheetah_common::room::object::GameObjectId;

use crate::clients::registry::ClientId;
use crate::ffi::command::send_command;
pub use crate::ffi::execute_with_client;

#[no_mangle]
pub extern "C" fn create_object(client_id: ClientId, template: u16, access_group: u64, result: &mut GameObjectId) -> u8 {
	execute_with_client(client_id, |client| {
		let game_object_id = client.create_game_object(template, access_group)?;
		*result = game_object_id;
		Ok(())
	})
}

#[no_mangle]
pub extern "C" fn created_object(client_id: ClientId, object_id: &GameObjectId, room_owner: bool, singleton_key: &Buffer) -> u8 {
	let singleton_key = (singleton_key.len > 0).then(|| *singleton_key);
	send_command(client_id, C2SCommand::CreatedGameObject(C2SCreatedGameObject::new(*object_id, room_owner, singleton_key).into()))
}

#[no_mangle]
pub extern "C" fn delete_object(client_id: ClientId, object_id: &GameObjectId) -> u8 {
	send_command(client_id, C2SCommand::Delete(*object_id))
}
