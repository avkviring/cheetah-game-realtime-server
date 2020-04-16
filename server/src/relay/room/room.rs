use crate::relay::room::clients::Clients;
use crate::relay::room::events::EventsCollector;
use crate::relay::room::objects::Objects;

pub type ClientId = u16;
pub type LocalObjectId = u32;
pub type GlobalObjectId = u64;


/// Описание комнаты
/// Комната - совокупность всех игроков, например битва.
/// Комнату создается по команде с match making.
/// При создании необходимо указать список ожидаемых пользователей.
pub struct Room {
	/// коллектор событий для отправки на клиент
	pub events_collector: EventsCollector,
	/// клиенты
	pub clients: Clients,
	/// список игровых объектов в комнате
	pub objects: Objects,
}

impl Room {
	pub fn new() -> Self {
		Room {
			events_collector: Default::default(),
			clients: Default::default(),
			objects: Default::default(),
		}
	}
}