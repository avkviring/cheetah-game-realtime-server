extern crate typenum;

use std::collections::{HashMap, HashSet};

use bit_array::BitArray;
use typenum::{U64, Unsigned};

use crate::relay::network::client::ClientStream;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::room::Room;

/// Ожидаемый клиент
pub struct ClientConfiguration {
    /// уникальный идентификатор клиента в рамках комнаты
    pub id: u16,
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

/// Группа пользователей
pub struct UsersGroup<'a> {
    users: Vec<&'a Client>,
}


impl ClientConfiguration {
    fn stub() -> ClientConfiguration {
        ClientConfiguration {
            id: 0,
            hash: "".to_string(),
            groups: AccessGroups::new(),
        }
    }
}

impl Client {
    pub(crate) fn stub() -> Client {
        Client {
            configuration: ClientConfiguration::stub(),
            stream: ClientStream::stub(),
        }
    }
}