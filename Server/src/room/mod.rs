use std::cell::RefCell;
use std::rc::Rc;

use indexmap::map::{IndexMap, MutableKeys};

use cheetah_relay_common::commands::hash::HashValue;
use cheetah_relay_common::constants::ClientId;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::ClientGameObjectId;

use crate::network::s2c::S2CCommandCollector;
use crate::room::client::{Client, Clients};
use crate::room::object::GameObject;
use crate::room::object::id::ServerGameObjectId;

pub mod command;
pub mod client;
pub mod object;
pub mod thread;
pub mod request;

/// Описание комнаты
/// Комната - совокупность всех игроков, например битва.
/// Комната создается по команде с mm сервера.
/// При создании необходимо указать список ожидаемых пользователей.
pub struct Room {
    ///
    /// true - создавать клиент даже если он не в списке ожидающих
    ///
    pub auto_create_client: bool,

    pub hash: HashValue,
    /// клиенты
    pub clients: Rc<RefCell<Clients>>,
    /// список игровых объектов в комнате
    objects: IndexMap<ServerGameObjectId, GameObject>,

    collector: Rc<RefCell<S2CCommandCollector>>,
}


#[derive(Debug)]
pub enum GameObjectCreateErrors {
    AlreadyExists(ServerGameObjectId)
}


impl Room {
    pub fn new(hash_value: HashValue, auto_create_client: bool) -> Self {
        let clients = Rc::new(RefCell::new(Clients::default()));
        Room {
            auto_create_client,
            hash: hash_value,
            clients: clients.clone(),
            objects: IndexMap::with_capacity(100),
            collector: Rc::new(RefCell::new(S2CCommandCollector::new(clients.clone()))),
        }
    }


    ///
    /// Получение игрового объекта с проверкой прав доступа
    /// TODO - добавить проверку прав
    ///
    pub fn get_object_with_check_access(&mut self,
                                        client: &Client,
                                        object_id: &ClientGameObjectId) ->
                                        Option<&mut GameObject> {
        let object_id = ServerGameObjectId::new(Option::Some(client.configuration.id), object_id);
        match self.objects.get_full_mut2(&object_id) {
            Some((pos, _, object)) => { Option::Some(object) }
            None => {
                log::error!("game object not found {:?} {:?}", client, object_id);
                Option::None
            }
        }
    }


    /// Присоединение клиента к комнате
    /// Хеш клиента должен быть в списке ожидающих клиентов
    pub fn client_connect(&mut self, client_hash: &HashValue) {
        // let result = self
        //     .clients
        //     .waiting_clients.remove(client_hash)
        //     .map(|client_configuration| {
        //         let id = client_configuration.id;
        //         let client = Rc::new(
        //             Client {
        //                 configuration: client_configuration
        //             });
        //
        //         self.clients
        //             .clients
        //             .insert(
        //                 id,
        //                 client.clone());
        //
        //         client
        //     });
        //
        // match result {
        //     None => {
        //         if self.auto_create_client {
        //             self.add_client_to_waiting_list(client_hash, AccessGroups::from(std::u64::MAX));
        //             self.client_connect(client_hash)
        //         } else {
        //             Result::Err(ClientNotInWaitingList)
        //         }
        //     }
        //     Some(client) => {
        //         Result::Ok(client)
        //     }
        // }
    }

    ///
    /// Добавить ожидающего клиента
    ///
    pub fn add_client_to_waiting_list(&mut self, hash: &HashValue, groups: AccessGroups) -> ClientId {
        panic!();
        // let client_id = self.clients.get_next_client_id();
        // let configuration = ClientConfiguration {
        //     id: client_id,
        //     hash: hash.clone(),
        //     groups,
        // };
        // self.clients.waiting_clients.insert(hash.clone(), configuration);
        // client_id
    }


    ///
    /// разрыв связи с пользователем
    /// окончательный
    /// повторный коннект обеспечивает сетевая часть
    ///
    pub fn client_disconnect(&mut self, client: &Client) {
        panic!()
        // let option = self.clients.clients.remove(&client.configuration.id);
        // if option.is_some() {
        //     let objects = self.objects.get_objects_by_owner(ServerOwner::Client(client.configuration.id));
        //     objects.iter().for_each(|o| {
        //         let o = o.clone();
        //         let o = &*o;
        //         let o = o.borrow();
        //         self.delete_game_object(&o);
        //     });
        //     self.listener.on_client_disconnect(client);
        // }
        // option
    }
}