use cheetah_relay_common::network::hash::HashValue;

use crate::room::clients::Clients;
use crate::room::listener::CompositeRoomListener;
use crate::room::objects::Objects;

pub mod clients;
pub mod objects;
pub mod listener;
pub mod thread;
pub mod request;

/// Описание комнаты
/// Комната - совокупность всех игроков, например битва.
/// Комната создается по команде с mm сервера.
/// При создании необходимо указать список ожидаемых пользователей.
pub struct Room {
	pub hash: HashValue,
	/// клиенты
	pub clients: Clients,
	/// список игровых объектов в комнате
	pub objects: Objects,
	/// обработчик событий игровой комнаты
	pub listener: CompositeRoomListener,
}

impl Room {
	pub fn new(hash_value: HashValue) -> Self {
		Room {
			hash: hash_value,
			listener: Default::default(),
			clients: Default::default(),
			objects: Default::default(),
		}
	}
}