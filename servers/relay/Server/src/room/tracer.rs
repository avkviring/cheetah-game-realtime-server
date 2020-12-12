use cheetah_relay_common::commands::command::{C2SCommand, S2CCommand};
use cheetah_relay_common::room::UserPublicKey;

use crate::room::RoomId;

///
/// Вывод отладочной информации по командам с клиента/сервера с учетом правил фильтрации.
/// Для отображения информации используется log::info
///
#[derive(Debug)]
pub struct Tracer {}

impl Tracer {
	///
	/// Создать трейсер для отображения всех событий
	///
	pub fn new_with_allow_all() -> Self {
		Self {}
	}

	pub fn on_s2c_command(&self, room_id: RoomId, user_public_key: UserPublicKey, command: &S2CCommand) {
		log::info!("[room({:?})] s -> u({:?}) {:?}", room_id, user_public_key, command);
	}
	pub fn on_c2s_command(&self, room_id: RoomId, user_public_key: UserPublicKey, command: &C2SCommand) {
		log::info!("[room({:?})] u({:?}) -> s {:?}", room_id, user_public_key, command);
	}
}
