use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::rc::Rc;

use cheetah_relay_common::commands::hash::HashValue;
use cheetah_relay_common::constants::ClientId;
use cheetah_relay_common::room::access::AccessGroups;

#[derive(Debug)]
pub struct Clients {
    /// список клиентов
    pub clients: HashMap<ClientId, Rc<Client>>,
    /// генератор идентификатора пользователя
    pub client_id_generator: ClientId,
    /// список ожидаемых клиентов
    pub waiting_clients: HashMap<HashValue, ClientConfiguration>,
}

/// Ожидаемый клиент
#[derive(Debug, PartialEq)]
pub struct ClientConfiguration {
    /// уникальный идентификатор клиента в рамках комнаты
    pub id: ClientId,
    /// авторизационный хеш
    pub hash: HashValue,
    /// группы
    pub groups: AccessGroups,
}

/// Клиент в комнате
#[derive(Debug, PartialEq)]
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
            client_id_generator: 128, // резерв
            waiting_clients: Default::default(),
        }
    }
}

