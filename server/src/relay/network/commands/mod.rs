use bytes::Bytes;

use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;

pub mod create_game_object;
pub mod delete_game_object;

/// Декодер входящей команды
pub trait CommandDecoder {
    /// идентификатор команды
    const COMMAND_ID: u8;

    /// Декодирование команды
    fn decode(bytes: &mut Bytes) -> Option<Box<dyn ClientCommandExecutor>>;
}


/// Интерфейс команды с клиента
pub trait ClientCommandExecutor {
    /// Выполнить команду
    fn execute(&self, client: &Client, room: &mut Room);
}