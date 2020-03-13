use std::any::Any;

use bytebuffer::ByteBuffer;

use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;
use traitcast::TraitcastFrom;

pub mod create_game_object;
pub mod delete_game_object;

/// Декодер входящей команды
pub trait ClientCommandDecoder {
    /// идентификатор команды
    const COMMAND_ID: u8;

    /// Декодирование команды
    /// bytes - массив байт, из которого будет прочитана информация
    /// если результат Option::None то указатель данных в bytes будет сброшен в начало
    fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn ClientCommandExecutor>>;
}


/// Интерфейс команды с клиента
pub trait ClientCommandExecutor : TraitcastFrom {
    /// Выполнить команду
    fn execute(&self, client: &Client, room: &mut Room);
}