use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, Ref};
use std::ops::Deref;
use std::rc::Rc;

use bytes::{Buf, Bytes};

use crate::relay::network::commands::{ClientCommandExecutor, CommandDecoder};
use crate::relay::network::commands::create_game_object::CreateGameObject;
use crate::relay::network::commands::delete_game_object::DeleteGameObject;
use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;

/// Декодирование сетевого потока в набор команд
struct Decoder {
    client: Rc<Client>,
    commands: Vec<Box<dyn ClientCommandExecutor>>,
}


impl Decoder {
    fn new(client: Rc<Client>) -> Decoder {
        Decoder {
            client,
            commands: Default::default(),
        }
    }

    /// декодирование потока
    /// return true - если есть команды для выполнения
    fn decode(&mut self, bytes: &mut Bytes) -> bool {
        // TODO - организовать цикл
        // TODO - организовать накопление данных
        while bytes.has_remaining() {
            let slice_for_read = &mut bytes.slice(0..bytes.len());
            let command_code = slice_for_read.get_u8();
            let command: Option<Box<dyn ClientCommandExecutor>> = match command_code {
                CreateGameObject::COMMAND_ID => CreateGameObject::decode(slice_for_read),
                DeleteGameObject::COMMAND_ID => DeleteGameObject::decode(slice_for_read),
                _ => Option::None
            };

            if command.is_some() {
                self.commands.push(command.unwrap());
                bytes.advance(bytes.remaining() - slice_for_read.remaining());
            }
        };
        return self.commands.len() > 0;
    }

    /// выполнить входящие команды
    fn execute(&mut self, room: &mut Room) {
        for command in self.commands.iter() {
            let mut rc = self.client.clone();
            command.execute(rc.borrow(), room)
        }
        self.commands.clear()
    }
}