use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::ops::Shl;

use bit_array::BitArray;
use typenum::U64;

use crate::relay::network::Connector;
use crate::relay::room::clients::{Client, ClientConfiguration};
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::object::{GameObject, Objects};

/// Описание комнаты
/// Комната - совокупность всех игроков, например битва.
/// Комнату создается по команде с match making.
/// При создании необходимо указать список ожидаемых пользователей.
pub struct Room {
    /// список ожидаемых клиентов
    pub waiting_clients: Vec<ClientConfiguration>,
    /// клиенты
    clients: HashMap<u16, Client>,
    /// список игровых объектов в комнате
    pub objects: Objects,
    /// генератор идентификатора пользователя
    client_id_generator: u16,
}


impl<'a> Room {
    pub fn new() -> Self {
        Room {
            waiting_clients: Default::default(),
            clients: Default::default(),
            objects: Default::default(),
            client_id_generator: Default::default(),
        }
    }

    /// Присоединение клиениа к комнате
    /// Хеш клиента должен быть в списке ожидающих клиентов
    pub fn connect(&mut self, client_hash: String) -> Result<(), ()> {
        let result =
            self
                .waiting_clients
                .iter()
                .position(|x| x.hash == client_hash)
                .map(|position| self.waiting_clients.remove(position))
                .ok_or(());


        return match result {
            Ok(client_configuration) => {
                self.clients.insert(
                    client_configuration.id,
                    Client {
                        configuration: client_configuration
                    });
                Result::Ok(())
            }
            Err(_) => {
                Result::Err(())
            }
        };
    }

    /// Создание клиентского игрового объекта
    /// owner - идентификатор клиента
    /// local_object_id - идентификатор объекта в рамках клиента
    /// groups - список групп
    pub fn create_client_game_object(&mut self, owner: u16, local_object_id: u32, groups: Vec<u8>) -> u64 {
        let client = self.get_client(owner).unwrap();


        let objectGroups = if groups.is_empty() {
            AccessGroups::new_from_groups(&client.configuration.groups)
        } else {
            AccessGroups::new_from_vec(groups)
        };

        let object = GameObject::new(owner, local_object_id, objectGroups);
        let id = object.id;
        self.objects.insert(object);
        return id;
    }

    pub fn get_client(&self, client: u16) -> Option<&Client> {
        return self.clients.get(&client);
    }

    pub fn get_clients(&self) -> Values<'_, u16, Client> {
        return self.clients.values();
    }

    /// Добавить ожидающего клиента
    pub fn add_waiting_client(&mut self, hash: &str, groups: Vec<u8>) -> u16 {
        let client_id = self.generate_client_id();
        let configuration = ClientConfiguration {
            id: client_id,
            hash: hash.to_string(),
            groups: AccessGroups::new_from_vec(groups),
        };
        self.waiting_clients.push(configuration);
        return client_id;
    }

    fn generate_client_id(&mut self) -> u16 {
        self.client_id_generator += 1;
        return self.client_id_generator;
    }
}