use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::rc::Rc;

use cheetah_relay_common::constants::ClientId;
use cheetah_relay_common::room::access::AccessGroups;

use crate::room::clients::ClientConnectError::ClientNotInWaitingList;
use crate::room::listener::RoomListener;
use crate::room::objects::owner::Owner;
use crate::room::room::Room;
use cheetah_relay_common::network::hash::HashValue;

pub struct Clients {
	/// список клиентов
	pub clients: HashMap<ClientId, Rc<Client>>,
	/// генератор идентификатора пользователя
	pub client_id_generator: ClientId,
	/// список ожидаемых клиентов
	pub waiting_clients: HashMap<HashValue, ClientConfiguration>,
}

/// Ожидаемый клиент
#[derive(Debug)]
pub struct ClientConfiguration {
	/// уникальный идентификатор клиента в рамках комнаты
	pub id: ClientId,
	/// авторизационный хеш
	pub hash: HashValue,
	/// группы
	pub groups: AccessGroups,
}

/// Клиент в комнате
#[derive(Debug)]
pub struct Client {
	/// конфигурация клиента
	pub configuration: ClientConfiguration,
}

#[derive(Debug)]
pub enum ClientConnectError {
	ClientNotInWaitingList
}

impl Clients {
	pub fn get_next_client_id(&mut self) -> ClientId {
		self.client_id_generator += 1;
		self.client_id_generator
	}
	pub fn get_client(&self, client: u16) -> Option<&Rc<Client>> {
		self.clients.get(&client)
	}
	pub fn get_clients(&self) -> Values<'_, u16, Rc<Client>> {
		self.clients.values()
	}
}

impl Default for Clients {
	fn default() -> Self {
		Clients {
			clients: Default::default(),
			client_id_generator: Default::default(),
			waiting_clients: Default::default(),
		}
	}
}

impl Room {
	/// Присоединение клиента к комнате
	/// Хеш клиента должен быть в списке ожидающих клиентов
	pub fn client_connect(&mut self, client_hash: &HashValue) -> Result<Rc<Client>, ClientConnectError> {
		self
			.clients
			.waiting_clients.remove(client_hash)
			.ok_or(ClientNotInWaitingList)
			.map(|client_configuration| {
				let id = client_configuration.id;
				let client = Rc::new(
					Client {
						configuration: client_configuration
					});
				
				self.clients
					.clients
					.insert(
						id,
						client.clone());
				
				self.listener.on_client_connect(&client.clone(), &self.objects);
				client
			})
	}
	
	/// Добавить ожидающего клиента
	pub fn add_client_to_waiting_list(&mut self, hash: &HashValue, groups: AccessGroups) -> ClientId {
		let client_id = self.clients.get_next_client_id();
		let configuration = ClientConfiguration {
			id: client_id,
			hash: hash.clone(),
			groups,
		};
		self.clients.waiting_clients.insert(hash.clone(), configuration);
		client_id
	}
	
	
	/// разрыв связи с пользователем
	/// окончательный
	/// повторный коннект обеспечивает сетевая часть
	pub fn client_disconnect(&mut self, client: &Client) -> Option<Rc<Client>> {
		let option = self.clients.clients.remove(&client.configuration.id);
		if option.is_some() {
			let objects = self.objects.get_objects_by_owner(Owner::new_owner(client));
			objects.iter().for_each(|o| {
				let o = o.clone();
				let o = &*o;
				let o = o.borrow();
				self.delete_game_object(&o);
			});
			self.listener.on_client_disconnect(client);
		}
		option
	}
}