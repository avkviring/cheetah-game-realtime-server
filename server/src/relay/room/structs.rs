use std::collections::{HashMap, HashSet};

use crate::relay::network::Connector;

/// Ожидаемый в комнате пользователь
pub struct UserAuth {
    /// авторизационный хеш пользователя
    pub hash: String
}

/// Список пользователей в комнате
pub struct ConnectedUser<'a> {
    pub user_auth: UserAuth,
    pub connector: &'a Connector,
}

/// Группа пользователей
pub struct UsersGroup<'a> {
    users: Vec<&'a ConnectedUser<'a>>,
    objects: HashSet<&'a GameObject>,
}

/// игровой объект
pub struct GameObject {
    id: u32
}