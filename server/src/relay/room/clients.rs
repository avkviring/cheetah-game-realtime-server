extern crate typenum;

use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::rc::Rc;

use crate::relay::network::client::ClientStream;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::objects::owner::Owner;
use crate::relay::room::room::{ClientId, Room};

pub struct Clients {
	/// список клиентов
	pub clients: HashMap<ClientId, Rc<Client>>,
	/// генератор идентификатора пользователя
	pub client_id_generator: ClientId,
	/// список ожидаемых клиентов
	pub waiting_clients: Vec<ClientConfiguration>,
}

/// Ожидаемый клиент
pub struct ClientConfiguration {
	/// уникальный идентификатор клиента в рамках комнаты
	pub id: ClientId,
	/// авторизационный хеш
	pub hash: String,
	/// группы
	pub groups: AccessGroups,
}

/// Клиент в комнате
pub struct Client {
	/// конфигурация клиента
	pub configuration: ClientConfiguration,
	/// сетевой поток клиента
	pub stream: ClientStream,
}

impl ClientConfiguration {
	fn stub(client_id: ClientId) -> ClientConfiguration {
		ClientConfiguration {
			id: client_id,
			hash: format!("{}", client_id),
			groups: AccessGroups::new(),
		}
	}
	
	fn stub_with_access_group(client_id: ClientId, group: Vec<u8>) -> ClientConfiguration {
		ClientConfiguration {
			id: client_id,
			hash: format!("{}", client_id),
			groups: AccessGroups::from(group),
		}
	}
}

impl Client {
	pub fn stub(client_id: u16) -> Client {
		Client {
			configuration: ClientConfiguration::stub(client_id),
			stream: ClientStream::stub(),
		}
	}
	
	pub fn stub_with_access_group(client_id: u16, groups: Vec<u8>) -> Client {
		Client {
			configuration: ClientConfiguration::stub_with_access_group(client_id, groups),
			stream: ClientStream::stub(),
		}
	}
}

impl Clients {
	pub fn get_next_client_id(&mut self) -> ClientId {
		self.client_id_generator += 1;
		return self.client_id_generator;
	}
	pub fn get_client(&self, client: u16) -> Option<&Rc<Client>> {
		return self.clients.get(&client);
	}
	pub fn get_clients(&self) -> Values<'_, u16, Rc<Client>> {
		return self.clients.values();
	}
}

impl Default for Clients {
	fn default() -> Self {
		return Clients {
			clients: Default::default(),
			client_id_generator: 0,
			waiting_clients: vec![],
		};
	}
}

impl Room {
	/// Присоединение клиента к комнате
	/// Хеш клиента должен быть в списке ожидающих клиентов
	pub fn client_connect(&mut self, client_hash: String) -> Result<Rc<Client>, ()> {
		let result =
			self
				.clients
				.waiting_clients
				.iter()
				.position(|x| x.hash == client_hash)
				.map(|position| self.clients.waiting_clients.remove(position))
				.ok_or(());
		
		
		return match result {
			Ok(client_configuration) => {
				let id = client_configuration.id;
				let client = Rc::new(
					Client {
						configuration: client_configuration,
						stream: ClientStream::stub(),
					});
				
				self.clients
					.clients
					.insert(
						id,
						client.clone());
				
				Result::Ok(client.clone())
			}
			Err(_) => {
				Result::Err(())
			}
		};
	}
	
	/// Добавить ожидающего клиента
	pub fn add_client_to_waiting_list(&mut self, hash: String, groups: AccessGroups) -> ClientId {
		let client_id = self.clients.get_next_client_id();
		let configuration = ClientConfiguration {
			id: client_id,
			hash,
			groups,
		};
		self.clients.waiting_clients.push(configuration);
		return client_id;
	}
	
	
	/// разрыв связи с пользователем
	/// окончательный
	/// повторный коннект обеспечивает сетевая часть
	pub fn client_disconnect(&mut self, client: &Client) -> Option<Rc<Client>> {
		let option = self.clients.clients.remove(&client.configuration.id);
		if option.is_some() {
			self.objects.delete_objects_by_owner(Owner::new_owner(client))
		}
		return option;
	}
}