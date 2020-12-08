use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;

use crate::ffi::{execute_with_client, GameObjectIdFFI};

/// Зарегистрировать обработчик загрузки нового игрового объекта
#[no_mangle]
pub extern "C" fn set_create_object_listener(listener: extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, template: u16)) -> bool {
	execute_with_client(|client| {
		client.register_create_object_listener(listener);
	})
	.is_ok()
}

#[no_mangle]
pub extern "C" fn create_object(template: u16, access_group: u64, result: &mut GameObjectIdFFI) -> bool {
	execute_with_client(|client| {
		let game_object_id = client.create_game_object(template, access_group);
		*result = game_object_id;
	})
	.is_ok()
}
