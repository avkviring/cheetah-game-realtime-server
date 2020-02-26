use std::collections::HashMap;

use crate::relay::network::Connector;
use crate::relay::room::structs::{ConnectedUser, GameObject, UserAuth, UsersGroup};

/// Описание комнаты
/// Комната - совокупность всех игроков, например битва.
/// Комнату создается по команде с match making.
/// При создании необходимо указать список ожидаемых пользователей.
pub struct Room<'a> {
    /// список ожидаемых пользователей
    pub waiting_users: Vec<UserAuth>,
    /// список пользователей в комнате
    pub users: Vec<ConnectedUser<'a>>,
    /// список групп пользователей
    pub groups: HashMap<String, UsersGroup<'a>>,
    /// список данных по объектам
    pub objects: HashMap<u32, GameObject>,
}


impl<'a> Room<'a> {
    pub fn new(waiting_users: Vec<UserAuth>) -> Room<'a> {
        Room {
            waiting_users,
            users: vec![],
            groups: Default::default(),
            objects: Default::default(),
        }
    }

    pub fn connect(&mut self, hash: String, connector: &'a mut Connector) -> Result<(), ()> {
        let p =
            self
                .waiting_users
                .iter()
                .position(|x| x.hash == hash)
                .map(|position| self.waiting_users.remove(position))
                .ok_or(());


        return match p {
            Ok(user_auth) => {
                self.users.push(ConnectedUser {
                    user_auth,
                    connector,
                });
                Result::Ok(())
            }
            Err(_) => {
                Result::Err(())
            }
        };
    }
}