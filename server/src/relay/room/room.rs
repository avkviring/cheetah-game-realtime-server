use std::collections::HashMap;
use std::ops::Shl;

use bit_array::BitArray;
use typenum::U64;

use crate::relay::network::Connector;
use crate::relay::room::clients::{Client, ClientConfiguration};
use crate::relay::room::object::{GameObject, Objects};

/// Описание комнаты
/// Комната - совокупность всех игроков, например битва.
/// Комнату создается по команде с match making.
/// При создании необходимо указать список ожидаемых пользователей.
pub struct Room {
    /// список ожидаемых клиентов
    pub waiting_clients: Vec<ClientConfiguration>,
    /// клиенты
    pub clients: Vec<Client>,
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
                self.clients.push(Client {
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
    pub fn create_client_game_object(&mut self, owner: u16, local_object_id: u32, groups: Vec<u8>) -> u64 {
        let object = GameObject::new(owner, local_object_id, groups);
        return object.id;
    }

    fn generate_client_id(&mut self) -> u16 {
        self.client_id_generator += 1;
        return self.client_id_generator;
    }

    /// Добавить ожидающего клиента
    pub fn add_waiting_client(&mut self, hash: &str, groups: Vec<u8>) {
        let mut groups_in_bit_set = BitArray::<u64, U64>::from_elem(false);
        for i in groups {
            groups_in_bit_set.set(i as usize, true)
        }
        let configuration = ClientConfiguration {
            id: self.generate_client_id(),
            hash: hash.to_string(),
            groups: groups_in_bit_set,
        };
        self.waiting_clients.push(configuration)
    }
}