use serde::{Deserialize, Serialize};

use crate::room::Room;

///
/// Выбор пользователя для входа
/// в режиме работы сервера без ММ
///
#[derive(Debug, Default)]
pub struct UserForEntranceSelector {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectedUserForEntrance {}

impl UserForEntranceSelector {
	pub fn select(&mut self, room: &Room) -> Option<SelectedUserForEntrance> {
		Option::None
	}
}
