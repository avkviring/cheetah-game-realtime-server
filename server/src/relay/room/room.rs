use crate::relay::room::clients::Clients;
use crate::relay::room::listener::CompositeRoomListener;
use crate::relay::room::objects::Objects;

pub type ClientId = u16;
pub type LocalObjectId = u32;
pub type GlobalObjectId = u64;


/// Описание комнаты
/// Комната - совокупность всех игроков, например битва.
/// Комнату создается по команде с match making.
/// При создании необходимо указать список ожидаемых пользователей.
pub struct Room {
	pub id: String,
	/// клиенты
	pub clients: Clients,
	/// список игровых объектов в комнате
	pub objects: Objects,
	/// обработчик событий игровой комнаты
	pub listener: CompositeRoomListener,
}

impl Room {
	pub fn new() -> Self {
		Room {
			id: Default::default(),
			listener: Default::default(),
			clients: Default::default(),
			objects: Default::default(),
		}
	}
}