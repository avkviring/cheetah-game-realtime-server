use cheetah_relay_common::constants::ClientId;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

///
/// Серверный идентификатор объекта
/// отличается от клиентского только тем что owner не может быть текущим клиентом
///
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ServerGameObjectId {
	pub id: u32,
	pub owner: ServerOwner,
}

///
/// Серверный владелец объекта
/// аналогичный ClientOwner кроме CurrentClient
/// так как для сервера нет понятия текущего клиента
///
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum ServerOwner {
	Root,
	Client(ClientId),
}

impl ServerGameObjectId {
	pub fn new(id: u32, owner: ServerOwner) -> Self {
		ServerGameObjectId {
			id,
			owner,
		}
	}
	///
	/// Конвертация клиентского id в серверный
	/// если current_client - None, то конвертация возможна только для Owner != ClientOwner::CurrentClient
	///
	pub fn from_client_object_id(current_client: Option<ClientId>, client_object_id: &ClientGameObjectId) -> ServerGameObjectId {
		ServerGameObjectId {
			id: client_object_id.id,
			owner: match client_object_id.owner {
				ClientOwner::Root => { ServerOwner::Root }
				ClientOwner::CurrentClient => { ServerOwner::Client(current_client.unwrap()) }
				ClientOwner::Client(client) => { ServerOwner::Client(client) }
			},
		}
	}
	///
	/// Конвертация серверного id в клиентский
	/// если current_client - None, то конвертация возможна только для Owner = Root
	///
	pub fn to_client_object_id(&self, current_client: Option<ClientId>) -> ClientGameObjectId {
		ClientGameObjectId {
			id: self.id,
			owner: match self.owner {
				ServerOwner::Root => { ClientOwner::Root }
				ServerOwner::Client(client) => {
					if client == current_client.unwrap() {
						ClientOwner::CurrentClient
					} else {
						ClientOwner::Client(client)
					}
				}
			},
		}
	}
}
