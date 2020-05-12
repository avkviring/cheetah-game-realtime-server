use crate::relay::room::clients::Clients;
use crate::relay::room::listener::CompositeRoomListener;
use crate::relay::room::objects::Objects;
use crate::relay::network::types::hash::HashValue;

pub type ClientId = u16;
pub type LocalObjectId = u32;
pub type GlobalObjectId = u64;

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